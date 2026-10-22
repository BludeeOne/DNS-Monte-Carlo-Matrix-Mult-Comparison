/* DNS implementation in C
 * Author: Erik Flores-Siemsen
 * Date: April 2026
 *  
 * Description: parallel dns resolution using requster resolver threads
 * I adapted from our previous assignment 
 * Requester threads pull hostnames from my hard coded list
 * and push them into a bounded buffer queue
 * resolvers pop hostnames and call dns lookup
 * results are written to an outfile
 * 
 * DNS is a good third system tester because unlike monte carlo and matrix mult.
 * dns is I/O bound. Threads spend most time waiting for netwrok responses or even waits
 * 
 * data sizes:
 * small: 50 hostnames
 * medium: 200 hostnames
 * large: 500 hostnames
 * 
 * How To Compile:
 *      gcc -O2 -o dns dns.c queue.c util.c -lpthread
 *
 * How To Run:
 *      ./dns <num_hosts> <num_threads> <outputfile>
 *      Example: ./dns 200 4 results.txt
 * 
 * 
 * ***ERIK NOTE CHECK DISCUSSION BOARD PROFESSOR GAVE A LINK TO A FILE WITH HOSTNAMES
 * remake code? my outs are kinda whatever they work
 * 
 * references in readme.md
 * */






#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <pthread.h>
#include <unistd.h>
#include <errno.h>
#include <time.h>

#include "queue.h"
#include "util.h"

#define MAX_RESOLVER_THREADS 64
#define MAX_NAME_LENGTH 1025
#define MAX_IP_LENGTH INET6_ADDRSTRLEN
#define QUEUE_SIZE 50
#define MAX_SLEEP_US 100

//hardcoded domains for testing
//i know its an ugly block but it work
static const char *HOSTNAMES[] = {
    "google.com",       "youtube.com",      "facebook.com",
    "amazon.com",       "wikipedia.org",    "twitter.com",
    "instagram.com",    "linkedin.com",     "reddit.com",
    "apple.com",        "microsoft.com",    "netflix.com",
    "github.com",       "stackoverflow.com","openai.com",
    "cloudflare.com",   "adobe.com",        "dropbox.com",
    "spotify.com",      "twitch.tv",        "discord.com",
    "slack.com",        "zoom.us",          "shopify.com",
    "paypal.com",       "ebay.com",         "pinterest.com",
    "tumblr.com",       "quora.com",        "medium.com",
    "nytimes.com",      "bbc.com",          "cnn.com",
    "theguardian.com",  "npr.org",          "espn.com",
    "weather.com",      "archive.org",      "wolframalpha.com",
    "nasa.gov",         "mit.edu",          "stanford.edu",
    "berkeley.edu",     "csuchico.edu",     "python.org",
    "rust-lang.org",    "golang.org",       "kernel.org",
    "debian.org",       "ubuntu.com"
};

static const int NUM_HOSTNAMES = sizeof(HOSTNAMES) / sizeof(HOSTNAMES[0]);

//shared queue and file
static queue g_queue;
static pthread_mutex_t g_queue_mutex = PTHREAD_MUTEX_INITIALIZER;
static pthread_mutex_t g_file_mutex = PTHREAD_MUTEX_INITIALIZER;

static volatile int g_requesters_done = 0;
static FILE *g_output_fp = NULL;

//shared hostname list
static char **g_hostnames = NULL;
static int g_num_hosts = 0;

//requester thread range
typedef struct {
    int start_idx;
    int end_idx;
} requester_arg_t;

//small random sleep to avoid fighting over mutex
void random_sleep_us() {
    usleep((unsigned int)(rand() % (MAX_SLEEP_US + 1)));
}

void *requester_thread(void *arg) {
    requester_arg_t *rarg = (requester_arg_t *)arg;

    for(int i = rarg->start_idx; i <= rarg->end_idx; i++) {
        char *hostname = strndup(g_hostnames[i], MAX_NAME_LENGTH - 1);

        if(!hostname) {
            fprintf(stderr, "ERROR: strndup failed: %s\n", strerror(errno));
            continue;
        }

        //wait until queue has room
        while(1) {
            pthread_mutex_lock(&g_queue_mutex);

            if(!queue_is_full(&g_queue)) {
                if(queue_push(&g_queue, hostname) == QUEUE_FAILURE) {
                    fprintf(stderr, "ERROR: queue_push failed\n");
                    free(hostname);
                }

                pthread_mutex_unlock(&g_queue_mutex);
                break;
            }

            pthread_mutex_unlock(&g_queue_mutex);
            random_sleep_us();
        }
    }

    return NULL;
}

void *resolver_thread(void *arg) {
    (void)arg;

    char ip[MAX_IP_LENGTH];

    while(1) {
        char *hostname = NULL;

        pthread_mutex_lock(&g_queue_mutex);

        if(!queue_is_empty(&g_queue)) {
            hostname = (char *)queue_pop(&g_queue);
        }
        else if(g_requesters_done) {
            pthread_mutex_unlock(&g_queue_mutex);
            break;
        }

        pthread_mutex_unlock(&g_queue_mutex);

        //nothing in queue yet
        if(!hostname) {
            random_sleep_us();
            continue;
        }

        ip[0] = '\0';

        if(dnslookup_multi(hostname, ip, sizeof(ip)) == UTIL_FAILURE) {
            fprintf(stderr, "WARNING: DNS lookup failed for hostname \"%s\"\n", hostname);
        }

        pthread_mutex_lock(&g_file_mutex);

        if(fprintf(g_output_fp, "%s,%s\n", hostname, ip) < 0) {
            fprintf(stderr, "ERROR: fprintf failed: %s\n", strerror(errno));
        }

        pthread_mutex_unlock(&g_file_mutex);

        free(hostname);
    }

    return NULL;
}

int main(int argc, char *argv[]) {

    if(argc != 4) {
        fprintf(stderr,
                "Usage: %s <num_hosts> <num_threads> <outputfile>\n"
                "Example: %s 200 4 results.txt\n",
                argv[0], argv[0]);
        return EXIT_FAILURE;
    }

    g_num_hosts = atoi(argv[1]);
    int num_threads = atoi(argv[2]);
    char *output_file = argv[3];

    if(g_num_hosts < 1) {
        fprintf(stderr, "ERROR: num_hosts must be >= 1\n");
        return EXIT_FAILURE;
    }

    if(num_threads < 1 || num_threads > MAX_RESOLVER_THREADS) {
        fprintf(stderr, "ERROR: num_threads must be between 1 and %d\n",
                MAX_RESOLVER_THREADS);
        return EXIT_FAILURE;
    }

    //build hostname list
    g_hostnames = (char **)malloc(g_num_hosts * sizeof(char *));

    if(!g_hostnames) {
        fprintf(stderr, "ERROR: malloc failed: %s\n", strerror(errno));
        return EXIT_FAILURE;
    }

    for(int i = 0; i < g_num_hosts; i++) {
        g_hostnames[i] = (char *)HOSTNAMES[i % NUM_HOSTNAMES];
    }

    //open output file
    g_output_fp = fopen(output_file, "w");

    if(!g_output_fp) {
        fprintf(stderr, "ERROR: cannot open output file \"%s\": %s\n",
                output_file, strerror(errno));
        free(g_hostnames);
        return EXIT_FAILURE;
    }

    //initialize queue
    if(queue_init(&g_queue, QUEUE_SIZE) == QUEUE_FAILURE) {
        fprintf(stderr, "ERROR: queue_init failed\n");
        fclose(g_output_fp);
        free(g_hostnames);
        return EXIT_FAILURE;
    }

    struct timespec start;
    struct timespec end;

    clock_gettime(CLOCK_MONOTONIC, &start);

    //start resolvers first
    pthread_t res_threads[MAX_RESOLVER_THREADS];

    for(int i = 0; i < num_threads; i++) {
        int rc = pthread_create(&res_threads[i], NULL, resolver_thread, NULL);

        if(rc != 0) {
            fprintf(stderr, "ERROR: pthread_create resolver %d failed: %s\n",
                    i, strerror(rc));

            for(int j = 0; j < i; j++) {
                pthread_cancel(res_threads[j]);
            }

            queue_cleanup(&g_queue);
            fclose(g_output_fp);
            free(g_hostnames);
            return EXIT_FAILURE;
        }
    }

    int num_requesters = num_threads;
    int hosts_per_requester = g_num_hosts / num_requesters;
    int remainder = g_num_hosts % num_requesters;

    pthread_t req_threads[MAX_RESOLVER_THREADS];
    requester_arg_t req_args[MAX_RESOLVER_THREADS];

    //split hostnames evenly across requesters
    for(int i = 0; i < num_requesters; i++) {
        req_args[i].start_idx = i * hosts_per_requester;
        req_args[i].end_idx = req_args[i].start_idx + hosts_per_requester - 1;

        if(i == num_requesters - 1) {
            req_args[i].end_idx += remainder;
        }

        int rc = pthread_create(&req_threads[i], NULL,
                                requester_thread, &req_args[i]);

        if(rc != 0) {
            fprintf(stderr, "ERROR: pthread_create requester %d failed: %s\n",
                    i, strerror(rc));
            num_requesters = i;
            break;
        }
    }

    //wait for requesters to finish
    for(int i = 0; i < num_requesters; i++) {
        pthread_join(req_threads[i], NULL);
    }

    pthread_mutex_lock(&g_queue_mutex);
    g_requesters_done = 1;
    pthread_mutex_unlock(&g_queue_mutex);

    //wait for resolvers
    for(int i = 0; i < num_threads; i++) {
        pthread_join(res_threads[i], NULL);
    }

    clock_gettime(CLOCK_MONOTONIC, &end);

    double elapsed = (end.tv_sec - start.tv_sec)
                   + (end.tv_nsec - start.tv_nsec) / 1e9;

    //cleanup
    queue_cleanup(&g_queue);
    fclose(g_output_fp);
    free(g_hostnames);

    printf("Hostnames: %d\n", g_num_hosts);
    printf("Threads: %d\n", num_threads);
    printf("Output file: %s\n", output_file);
    printf("Elapsed time: %.6f seconds\n", elapsed);

    return EXIT_SUCCESS;
}
