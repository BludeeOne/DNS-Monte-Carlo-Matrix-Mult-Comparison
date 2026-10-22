/*
 * multi-lookup.c
 * Author: Erik Flores-Siemsen
 *
 * Multi-process DNS lookup using shared memory.
 * Requester processes -> shared bounded buffer -> resolver processes
 * Resources:
 * https://man7.org/linux/man-pages/man2/mmap.2.html
 * https://man7.org/linux/man-pages/man3/pthread_mutexattr_setpshared.3p.html
 * https://man7.org/linux/man-pages/man3/pthread_condattr_setpshared.3p.html

 */

#include "multi-lookup.h"

shared_mem_t *init_shared_mem(void)
{
    shared_mem_t *shm = mmap(NULL, sizeof(shared_mem_t),
                             PROT_READ | PROT_WRITE,
                             MAP_SHARED | MAP_ANONYMOUS,
                             -1, 0);
    if (shm == MAP_FAILED) {
        fprintf(stderr, "ERROR: mmap failed: %s\n", strerror(errno));
        return NULL;
    }

    shm->head            = 0;
    shm->tail            = 0;
    shm->count           = 0;
    shm->requesters_done = 0;
    shm->num_requesters  = 0;

    pthread_mutexattr_t mattr;
    pthread_mutexattr_init(&mattr);
    pthread_mutexattr_setpshared(&mattr, PTHREAD_PROCESS_SHARED);
    pthread_mutex_init(&shm->mutex,      &mattr);

    // mutex for output file 
    pthread_mutex_init(&shm->file_mutex, &mattr);
    pthread_mutexattr_destroy(&mattr);

    // shared condition variables 
    pthread_condattr_t cattr;
    pthread_condattr_init(&cattr);
    pthread_condattr_setpshared(&cattr, PTHREAD_PROCESS_SHARED);
    pthread_cond_init(&shm->not_full,  &cattr);
    pthread_cond_init(&shm->not_empty, &cattr);
    pthread_condattr_destroy(&cattr);

    return shm;
}

void destroy_shared_mem(shared_mem_t *shm)
{
    pthread_mutex_destroy(&shm->mutex);
    pthread_mutex_destroy(&shm->file_mutex);
    pthread_cond_destroy(&shm->not_full);
    pthread_cond_destroy(&shm->not_empty);
    munmap(shm, sizeof(shared_mem_t));
}

void requester_process(const char *filename, shared_mem_t *shm)
{
    FILE *fp = fopen(filename, "r");
    if (!fp) {
        // bogus input
        fprintf(stderr, "ERROR: Cannot open input file \"%s\": %s\n",
                filename, strerror(errno));
        //still mark this requester as done so resolvers don't hang 
        pthread_mutex_lock(&shm->mutex);
        shm->requesters_done++;
        pthread_cond_broadcast(&shm->not_empty);
        pthread_mutex_unlock(&shm->mutex);
        return;
    }

    char hostname[MAX_NAME_LENGTH];

    while (fgets(hostname, sizeof(hostname), fp)) {
        hostname[strcspn(hostname, "\r\n")] = '\0';
        if (hostname[0] == '\0') continue;  

        pthread_mutex_lock(&shm->mutex);

        // wait on condvar while buffer is full 
        while (shm->count == QUEUE_SIZE) {
            pthread_cond_wait(&shm->not_full, &shm->mutex);
        }

        //push into buffer
        strncpy(shm->buffer[shm->tail], hostname, MAX_NAME_LENGTH - 1);
        shm->buffer[shm->tail][MAX_NAME_LENGTH - 1] = '\0';
        shm->tail  = (shm->tail + 1) % QUEUE_SIZE;
        shm->count++;

        pthread_cond_signal(&shm->not_empty);
        pthread_mutex_unlock(&shm->mutex);
    }

    if (ferror(fp)) {
        fprintf(stderr, "ERROR: I/O error reading \"%s\": %s\n",
                filename, strerror(errno));
    }

    fclose(fp);

    // mark this requester as done resolvers use count vs num_requesters 
    pthread_mutex_lock(&shm->mutex);
    shm->requesters_done++;
    pthread_cond_broadcast(&shm->not_empty);
    pthread_mutex_unlock(&shm->mutex);
}

void resolver_process(shared_mem_t *shm, FILE *outputfp)
{
    char hostname[MAX_NAME_LENGTH];
    char ipstr[MAX_IP_LENGTH];

    while (1) {
        pthread_mutex_lock(&shm->mutex);

        // wait while buffer empty and not all requesters finished 
        while (shm->count == 0 &&
               shm->requesters_done < shm->num_requesters) {
            pthread_cond_wait(&shm->not_empty, &shm->mutex);
        }

        // exit: buffer empty and all requesters done 
        if (shm->count == 0 &&
            shm->requesters_done >= shm->num_requesters) {
            pthread_mutex_unlock(&shm->mutex);
            break;
        }

        // pop from  buffer 
        strncpy(hostname, shm->buffer[shm->head], MAX_NAME_LENGTH - 1);
        hostname[MAX_NAME_LENGTH - 1] = '\0';
        shm->head  = (shm->head + 1) % QUEUE_SIZE;
        shm->count--;

        pthread_cond_signal(&shm->not_full);
        pthread_mutex_unlock(&shm->mutex);

        // dns lookup with bogus hostname
        ipstr[0] = '\0';
        if (dnslookup(hostname, ipstr, sizeof(ipstr)) == UTIL_FAILURE) {
            fprintf(stderr,
                    "WARNING: DNS lookup failed for hostname \"%s\"\n",
                    hostname);
        }

        // write under file mutex, if it wasnt multiple processes could betwrting at once
        pthread_mutex_lock(&shm->file_mutex);
        if (fprintf(outputfp, "%s,%s\n", hostname, ipstr) < 0) {
            fprintf(stderr, "ERROR: fprintf to output file failed: %s\n",
                    strerror(errno));
        }
        fflush(outputfp);
        pthread_mutex_unlock(&shm->file_mutex);
    }
}

int main(int argc, char *argv[])
{
    if (argc < 3) {
        fprintf(stderr,
                "Usage: %s <namefile1> [namefile2 ...] <outputfile>\n"
                "  Max input files    : %d\n"
                "  Resolver processes : %d - %d\n",
                argv[0], MAX_INPUT_FILES,
                MIN_RESOLVER_PROCESSES, MAX_RESOLVER_PROCESSES);
        return EXIT_FAILURE;
    }

    int    num_input_files = argc - 2;
    char **input_files     = &argv[1];
    char  *output_file     = argv[argc - 1];

    //safeguards for max inputs
    if (num_input_files > MAX_INPUT_FILES) {
        fprintf(stderr,
                "ERROR: Too many input files (%d). Maximum allowed: %d.\n",
                num_input_files, MAX_INPUT_FILES);
        return EXIT_FAILURE;
    }

    //bogus out check
    FILE *outputfp = fopen(output_file, "w");
    if (!outputfp) {
        fprintf(stderr, "ERROR: Cannot open output file \"%s\": %s\n",
                output_file, strerror(errno));
        return EXIT_FAILURE;
    }

    shared_mem_t *shm = init_shared_mem();
    if (!shm) {
        fclose(outputfp);
        return EXIT_FAILURE;
    }

    shm->num_requesters = num_input_files;

    int num_resolvers = num_input_files < MIN_RESOLVER_PROCESSES
                        ? MIN_RESOLVER_PROCESSES : num_input_files;
    if (num_resolvers > MAX_RESOLVER_PROCESSES)
        num_resolvers = MAX_RESOLVER_PROCESSES;

    pid_t res_pids[MAX_RESOLVER_PROCESSES];
    pid_t req_pids[MAX_INPUT_FILES];

    //fork resolvers first so requesters dont fill up the buffer before resolvers get made
    for (int i = 0; i < num_resolvers; i++) {
        pid_t pid = fork();
        if (pid < 0) {
            fprintf(stderr, "ERROR: fork (resolver %d) failed: %s\n",
                    i, strerror(errno));
            for (int j = 0; j < i; j++)
                kill(res_pids[j], SIGTERM);
            fclose(outputfp);
            destroy_shared_mem(shm);
            return EXIT_FAILURE;
        }
        if (pid == 0) {
            resolver_process(shm, outputfp);
            fclose(outputfp);
            exit(EXIT_SUCCESS);
        }
        res_pids[i] = pid;
    }

    // fork requester processes: one per input file 
    int started_requesters = 0;
    for (int i = 0; i < num_input_files; i++) {
        pid_t pid = fork();
        if (pid < 0) {
            fprintf(stderr, "ERROR: fork (requester %d) failed: %s\n",
                    i, strerror(errno));
            break;
        }
        if (pid == 0) {
            requester_process(input_files[i], shm);
            exit(EXIT_SUCCESS);
        }
        req_pids[i] = pid;
        started_requesters++;
    }

    //requester wit for all to be done
    for (int i = 0; i < started_requesters; i++) {
        if (waitpid(req_pids[i], NULL, 0) < 0) {
            fprintf(stderr, "ERROR: waitpid (requester %d) failed: %s\n",
                    i, strerror(errno));
        }
    }

   //resolver wait for all to be done
    for (int i = 0; i < num_resolvers; i++) {
        if (waitpid(res_pids[i], NULL, 0) < 0) {
            fprintf(stderr, "ERROR: waitpid (resolver %d) failed: %s\n",
                    i, strerror(errno));
        }
    }

    fclose(outputfp);
    destroy_shared_mem(shm);

    return EXIT_SUCCESS;
}