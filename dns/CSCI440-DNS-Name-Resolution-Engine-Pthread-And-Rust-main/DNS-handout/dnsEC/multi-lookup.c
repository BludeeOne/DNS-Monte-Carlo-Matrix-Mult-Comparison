//Author: Flores-Siemsen
//extra credit attempt


#include "multi-lookup.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <pthread.h>
#include <unistd.h>     //usleep and sysconf(ec)
#include <errno.h>

#include "queue.h"
#include "util.h"

static queue           g_queue;
//threads for hostnames from reading old file
static pthread_mutex_t g_queue_mutex = PTHREAD_MUTEX_INITIALIZER;
//threads to write into new file with hostnames and ids after dns lookup
static pthread_mutex_t g_file_mutex  = PTHREAD_MUTEX_INITIALIZER;


static volatile int g_requesters_done = 0;

static FILE *g_output_fp = NULL;    

//sleep random so everything doesnt collide when waking up 
void random_sleep_us(void)
{
    usleep((unsigned int)(rand() % (MAX_SLEEP_US + 1)));
}

/*create a series of threads pointer arguments. Read the input file, clean the hostnames, 
then use threads to try to push the hostnames into a queue that resolver threads then takes 
only one thread should be allowed to push at a time or we could corrupt our queue*/
void *requester_thread(void *arg)
{
    //
    requester_arg_t *rarg = (requester_arg_t *)arg;

    FILE *fp = fopen(rarg->filename, "r");
    if (!fp) {
        fprintf(stderr,
                "ERROR: Cannot open input file \"%s\": %s\n",
                rarg->filename, strerror(errno));
        return NULL;
    }

    char line[MAX_NAME_LENGTH];

    while (fgets(line, sizeof(line), fp) != NULL) {
        line[strcspn(line, "\r\n")] = '\0';

        if (line[0] == '\0')
            continue;

        char *hostname = strndup(line, MAX_NAME_LENGTH - 1);
        if (!hostname) {
            fprintf(stderr, "ERROR: strndup failed: %s\n", strerror(errno));
            continue;
        }

        while(1) {
            //only one thread can be inside at a time
            pthread_mutex_lock(&g_queue_mutex);
            if (!queue_is_full(&g_queue)) {
                if (queue_push(&g_queue, hostname) == QUEUE_FAILURE) {
                    fprintf(stderr, "ERROR: queue_push failed\n");
                    free(hostname);
                }
                //if queue isnt full then we can push the hostname and leave
                pthread_mutex_unlock(&g_queue_mutex);
                break;
            }
            //unlock and let thread leave, sleep random and then try again
            pthread_mutex_unlock(&g_queue_mutex);
            random_sleep_us();   // queue full go night night
        }
    }

    if (ferror(fp)) {
        fprintf(stderr,
                "ERROR: I/O error while reading \"%s\": %s\n",
                rarg->filename, strerror(errno));
    }

    fclose(fp);
    return NULL;
}


void *resolver_thread(void *arg)
{
    (void)arg;   //stops warnings

    char ip[1024]; //address holder

    while(1) {
        char *hostname = NULL;

        pthread_mutex_lock(&g_queue_mutex);
        if (!queue_is_empty(&g_queue)) {
            hostname = (char *)queue_pop(&g_queue);
        } 
        else if (g_requesters_done) {
            //nothing left in queue 
            pthread_mutex_unlock(&g_queue_mutex);
            break;
        }
        pthread_mutex_unlock(&g_queue_mutex);

        if (!hostname) {
            //wait your turn
            random_sleep_us();
            continue;
        }


        ip[0] = '\0';
        if (dnslookup_multi(hostname, ip, sizeof(ip)) == UTIL_FAILURE) {
        
            fprintf(stderr,
                    "WARNING: DNS lookup failed for hostname \"%s\"\n",
                    hostname);
        }

        pthread_mutex_lock(&g_file_mutex);
        if (fprintf(g_output_fp, "%s,%s\n", hostname, ip) < 0) {
            fprintf(stderr, "ERROR: fprintf to output file failed: %s\n",
                    strerror(errno));
        }
        pthread_mutex_unlock(&g_file_mutex);

        free(hostname); //free memory
    }

    return NULL;
}


int main(int argc, char *argv[])
{
    //argc: number of input files given
    //argv vector of the input files
    if (argc < 3) {
        fprintf(stderr,
                "Usage: %s <namefile1> [namefile2 ...] <outputfile>\n"
                "  Maximum input files : %d\n"
                "  Resolver threads    : %d - %d\n",
                argv[0],
                MAX_INPUT_FILES,
                MIN_RESOLVER_THREADS,
                MAX_RESOLVER_THREADS);
        return EXIT_FAILURE;
    }
//one file for output and one for program name, the rest are the actual input
    int    num_input_files = argc - 2; 
    char **input_files     = &argv[1];
    char  *output_file     = argv[argc - 1];

    if (num_input_files > MAX_INPUT_FILES) {
        fprintf(stderr,
                "ERROR: Too many input files (%d). Maximum allowed: %d.\n",
                num_input_files, MAX_INPUT_FILES);
        return EXIT_FAILURE;
    }

    g_output_fp = fopen(output_file, "w");
    if (!g_output_fp) {
        fprintf(stderr,
                "ERROR: Cannot open output file \"%s\": %s\n",
                output_file, strerror(errno));
        return EXIT_FAILURE;
    }

    if (queue_init(&g_queue, QUEUE_SIZE) == QUEUE_FAILURE) {
        fprintf(stderr, "ERROR: queue_init failed\n");
        fclose(g_output_fp);
        return EXIT_FAILURE;
    }
    //benchmark testing core vs hard code
    long num_cores = sysconf(_SC_NPROCESSORS_ONLN);
    //long num_cores = 16; 
    if (num_cores <1) num_cores = 1;
    int num_resolvers = (int)num_cores;
    if (num_resolvers < MIN_RESOLVER_THREADS)
        num_resolvers = MIN_RESOLVER_THREADS;
    if (num_resolvers > MAX_RESOLVER_THREADS)
        num_resolvers = MAX_RESOLVER_THREADS;


    int num_requesters = num_input_files;  /* exactly one thread per file  */

    pthread_t       *req_threads = calloc((size_t)num_requesters, sizeof(pthread_t));
    pthread_t       *res_threads = calloc((size_t)num_resolvers,  sizeof(pthread_t));
    requester_arg_t *req_args    = calloc((size_t)num_requesters, sizeof(requester_arg_t));

    if (!req_threads || !res_threads || !req_args) {
        fprintf(stderr, "ERROR: Memory allocation failed: %s\n", strerror(errno));
        free(req_threads);
        free(res_threads);
        free(req_args);
        queue_cleanup(&g_queue);
        fclose(g_output_fp);
        return EXIT_FAILURE;
    }

    //launch resolvers first so we dont fill up queue with requesters before ant resolvers get to the scene
    for (int i = 0; i < num_resolvers; i++) {
        int rc = pthread_create(&res_threads[i], NULL, resolver_thread, NULL);
        if (rc != 0) {
            //if failed in creating one release all in memory
            fprintf(stderr,
                    "ERROR: pthread_create (resolver %d) failed: %s\n",
                    i, strerror(rc));
            for (int j = 0; j < i; j++)
                pthread_cancel(res_threads[j]);
            free(req_threads);
            free(res_threads);
            free(req_args);
            queue_cleanup(&g_queue);
            fclose(g_output_fp);
            return EXIT_FAILURE;
        }
    }

    //creation of requester threads
    for (int i = 0; i < num_requesters; i++) {
        req_args[i].filename = input_files[i];
        int rc = pthread_create(&req_threads[i], NULL, requester_thread, &req_args[i]);

        if (rc != 0) {
            fprintf(stderr,
                    "ERROR: pthread_create (requester %d) failed: %s\n",
                    i, strerror(rc));
           
            num_requesters = i;
            break;
        }
    }

    for (int i = 0; i < num_requesters; i++) {
        int rc = pthread_join(req_threads[i], NULL);
        if (rc != 0) {
            fprintf(stderr,
                    "ERROR: pthread_join (requester %d) failed: %s\n",
                    i, strerror(rc));
        }
    }

    pthread_mutex_lock(&g_queue_mutex);
    g_requesters_done = 1;
    pthread_mutex_unlock(&g_queue_mutex);

    for (int i = 0; i < num_resolvers; i++) {
        int rc = pthread_join(res_threads[i], NULL);
        if (rc != 0) {
            fprintf(stderr,
                    "ERROR: pthread_join (resolver %d) failed: %s\n",
                    i, strerror(rc));
        }
    }

//cleanup   
    queue_cleanup(&g_queue);
    fclose(g_output_fp);
    free(req_threads);
    free(res_threads);
    free(req_args);

    return EXIT_SUCCESS;
}