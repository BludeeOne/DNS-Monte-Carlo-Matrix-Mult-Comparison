//multi-lookup.h
//author: Erik Flores-Siemsen

#ifndef MULTI_LOOKUP_H
#define MULTI_LOOKUP_H

#include <stdio.h>
#include <pthread.h>
#include "queue.h"

//limits 
#define MAX_INPUT_FILES      10     
#define MAX_RESOLVER_THREADS 10     
#define MIN_RESOLVER_THREADS 2     //rare case threads dont create, could potentially leave program hanging  
#define MAX_NAME_LENGTH      1025    
#define MAX_IP_LENGTH        INET6_ADDRSTRLEN

#define QUEUE_SIZE           50      /* bounded-buffer capacity              */
#define MAX_SLEEP_US         100     /* requester max time to spend sleeping while waiting for space in queue     */

typedef struct {
    char *filename;                 
} requester_arg_t;

//prototypes

void *requester_thread(void *arg);
/*requester threads read the file and clean the usernames 
i.e www.google.com\r becomes www.google.com
after cleaning each username they then attempt to push a mutex queue
use mutex because we have mulitple threads and if they all pushed and popped at once we could corrupt data or memory leak
once all hostnames are read,cleaned, and pushed to queue requesters leave and close file */


void *resolver_thread(void *arg);
/* resolver threads take hostnames put in the queue and take them out to dns lookup, once each name has a ip address attached
they are then written to an outfile within a seperate mutex to keep threads reading, popping, and writing one thing at a time
once all requesters have left and the queue is cleared the resolvers wait until everything is written to the outfile before freeing
and returning*/


void random_sleep_us(void);
//night night 
//sleep is random because if my threads woke up at the same time they could collide

#endif 