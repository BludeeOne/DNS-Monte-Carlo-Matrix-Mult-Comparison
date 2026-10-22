/* Monte Carlo algorithim coded in C
 * AUthor: Erik Flroes-Siemsen
 * Date: May 2026 
 *  
 * Description: This file uses the Monte carlo method to repeatedly
 * randomly generate numbers to estimate pi. 
 * First create a unit square and then a quarter circle inside of it,
 * By generating random points within the square and counting how many 
 * land within the circle and taking the ratio of inside/total we get pi/4. 
 * Multiplying that number by 4 would give you pi. 
 * 
 * Compile is inside the Readme
 * references is also in the readme 
 *
*/




#include <stdio.h>
#include <stdlib.h>
#include <pthread.h>
#include <math.h>
#include <time.h>

#define MAX_THREADS 64

long long global_inside = 0;    //points inside circle
pthread_mutex_t lock;

typedef struct {
    long long num_points;
    int thread_id;
} ThreadArgs;

double randomNumber() {
    return rand() / (double)RAND_MAX;
}

void *pi_thread(void *arg) {

    ThreadArgs *args = (ThreadArgs *)arg;

    long long points = args->num_points;
    int tid = args->thread_id;

    //to simulate a large random number take current time and multiply the thread id
    // by a large prime number (1,000,003)
    srand((unsigned int)time(NULL) ^ (unsigned int)(tid * 1000003));

    long long local_inside = 0;

    for(long long i = 0; i < points; i++) {

        double x = randomNumber();
        double y = randomNumber();

        if((x * x) + (y * y) <= 1.0) {
            local_inside++;
        }
    }

    pthread_mutex_lock(&lock);
    global_inside += local_inside;
    pthread_mutex_unlock(&lock);

    pthread_exit(NULL);
}

int main(int argc, char *argv[]) {

    if(argc != 3) {
        fprintf(stderr, "Usage: %s <num_points> <num_threads>\n", argv[0]);
        fprintf(stderr, "Example: %s 1000000 4\n", argv[0]);
        return EXIT_FAILURE;
    }

    long long num_points = atoll(argv[1]);
    int num_threads = atoi(argv[2]);

    if(num_threads < 1 || num_threads > MAX_THREADS) {
        fprintf(stderr, "Error: num_threads must be between 1 and %d\n", MAX_THREADS);
        return EXIT_FAILURE;
    }

    if(num_points < num_threads) {
        fprintf(stderr, "Error: num_points must be >= num_threads\n");
        return EXIT_FAILURE;
    }

    struct timespec start, end;
    clock_gettime(CLOCK_MONOTONIC, &start);

    pthread_t threads[MAX_THREADS];
    ThreadArgs args[MAX_THREADS];

    pthread_mutex_init(&lock, NULL);

    long long points_per_thread = num_points / num_threads;
    long long remainder = num_points % num_threads;

    //split work across threads
    for(int i = 0; i < num_threads; i++) {

        args[i].thread_id = i;
        args[i].num_points = points_per_thread +
                             (i == num_threads - 1 ? remainder : 0);

        if(pthread_create(&threads[i], NULL,
                          pi_thread, &args[i]) != 0) {

            fprintf(stderr, "Error: failed to create thread %d\n", i);
            return EXIT_FAILURE;
        }
    }

    //wait for threads
    for(int i = 0; i < num_threads; i++) {
        pthread_join(threads[i], NULL);
    }

    pthread_mutex_destroy(&lock);

    clock_gettime(CLOCK_MONOTONIC, &end);

    double elapsed = (end.tv_sec - start.tv_sec)
                   + (end.tv_nsec - start.tv_nsec) / 1e9;

    double pi_estimate = 4.0 * (double)global_inside / (double)num_points;

    printf("Threads: %d\n", num_threads);
    printf("Points: %lld\n", num_points);
    printf("Inside circle: %lld\n", global_inside);
    printf("Pi estimate: %.9f\n", pi_estimate);
    printf("Actual pi: %.9f\n", M_PI);
    printf("Error: %.6f%%\n",
           fabs(pi_estimate - M_PI) / M_PI * 100.0);
    printf("Elapsed time: %.6f seconds\n", elapsed);

    return EXIT_SUCCESS;
}