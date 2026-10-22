/*
 * multi-lookup.h
 * Author: Erik Flores-Siemsen
 *
 * Multi-process DNS lookup using shared memory.
 */

#ifndef MULTI_LOOKUP_H
#define MULTI_LOOKUP_H

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>
#include <unistd.h>
#include <signal.h>
#include <pthread.h>
#include <sys/mman.h>
#include <sys/wait.h>
#include <arpa/inet.h>

#include "util.h"

//limits
#define MAX_INPUT_FILES         10
#define MAX_RESOLVER_PROCESSES  10
#define MIN_RESOLVER_PROCESSES  2
#define MIN_REQUESTER_PROCESSES 1
#define MAX_NAME_LENGTH         1025
#define MAX_IP_LENGTH           INET6_ADDRSTRLEN
#define QUEUE_SIZE              50

/*
 * shared_mem_t — lives in mmap shared memory so all processes can access it.
 * Forked processes get their own copy of normal memory, so shared data must
 * live here with PTHREAD_PROCESS_SHARED mutexes and condition variables.
 */
typedef struct {
    char buffer[QUEUE_SIZE][MAX_NAME_LENGTH]; 
    int  head;                                
    int  tail;                              
    int  count;             // number of items currently in buffer         
    int  requesters_done;   
    int  num_requesters;    

    pthread_mutex_t mutex;      // protects buffer, count, requesters_done 
    pthread_cond_t  not_full;   // requesters wait here when buffer full   
    pthread_cond_t  not_empty;  // resolvers wait here when buffer empty   
    pthread_mutex_t file_mutex; // protects output file writes             
} shared_mem_t;

//prototypes
// allocates and initialises shared memory with process-shared sync objects 
shared_mem_t *init_shared_mem(void);

// destroys sync objects and unmaps shared memory 
void destroy_shared_mem(shared_mem_t *shm);

// reads one input file, pushes hostnames onto bounded buffer 
void requester_process(const char *filename, shared_mem_t *shm);

// pops hostnames, resolves IPs, writes to output file 
void resolver_process(shared_mem_t *shm, FILE *outputfp);

#endif 