/* Matrix Multiplication in C
 * Author: Erik Flores-siemsen
 * Date: may 2026
 *  
 * Descrition: Parallel matrix multiplication using pthreads. 
 * This file/code multiplies two square N x N matrices 
 * A and B to produe result matrix C. 
 * This formula: C[i][j] = sum of A[i][k] * B[k][j] for k = 0..N-1
 * Each thread works on a contiguous range of rows of the result matrix C
 * and does it works independently. Therefore no mutex is needed because theyre writing
 * to their own regions within the matrix C
 * 
 * compile and references in README.md
 * 
 * data size benchmarks: 
 * 256
 * 512
 * 1024
 * 
 * 
 * */


#include <stdio.h>
#include <stdlib.h>
#include <pthread.h>
#include <time.h>
#include <math.h>

int N;
int num_threads;

double *A;
double *B;
double *C;

//thread row range
typedef struct {
    int start_row;
    int end_row;
    int thread_id;
} ThreadArgs;

void *matrix_thread(void *arg) {

    ThreadArgs *args = (ThreadArgs *)arg;

    int start = args->start_row;
    int end = args->end_row;

    for(int i = start; i <= end; i++) {

        for(int j = 0; j < N; j++) {

            double sum = 0.0;

            for(int k = 0; k < N; k++) {
                sum += A[i * N + k] * B[k * N + j];
            }

            C[i * N + j] = sum;
        }
    }

    pthread_exit(NULL);
}

int main(int argc, char *argv[]) {

    if(argc != 3) {
        fprintf(stderr,
                "Usage: %s <matrix_size> <num_threads>\n",
                argv[0]);

        fprintf(stderr,
                "Example: %s 512 4\n",
                argv[0]);

        return EXIT_FAILURE;
    }

    N = atoi(argv[1]);
    num_threads = atoi(argv[2]);

    if(N < 1) {
        fprintf(stderr, "ERROR: matrix_size must be >= 1\n");
        return EXIT_FAILURE;
    }

    if(num_threads < 1 || num_threads > 64) {
        fprintf(stderr,
                "ERROR: num_threads must be between 1 and 64\n");

        return EXIT_FAILURE;
    }

    if(N < num_threads) {
        fprintf(stderr,
                "ERROR: matrix_size must be >= num_threads\n");

        return EXIT_FAILURE;
    }

    //allocate matrices
    A = (double *)malloc(N * N * sizeof(double));
    B = (double *)malloc(N * N * sizeof(double));
    C = (double *)malloc(N * N * sizeof(double));

    if(!A || !B || !C) {
        fprintf(stderr, "ERROR: memory allocation failed\n");
        return EXIT_FAILURE;
    }

    //fill matrices with random values
    srand((unsigned int)time(NULL));

    for(int i = 0; i < N * N; i++) {
        A[i] = rand() / (double)RAND_MAX;
        B[i] = rand() / (double)RAND_MAX;
        C[i] = 0.0;
    }

    struct timespec start;
    struct timespec end;

    clock_gettime(CLOCK_MONOTONIC, &start);

    pthread_t threads[64];
    ThreadArgs args[64];

    int rows_per_thread = N / num_threads;
    int remainder = N % num_threads;

    //split rows across threads
    for(int i = 0; i < num_threads; i++) {

        args[i].thread_id = i;
        args[i].start_row = i * rows_per_thread;

        if(i == num_threads - 1) {
            args[i].end_row =
                args[i].start_row + rows_per_thread + remainder - 1;
        }
        else {
            args[i].end_row =
                args[i].start_row + rows_per_thread - 1;
        }

        if(pthread_create(&threads[i], NULL,
                          matrix_thread, &args[i]) != 0) {

            fprintf(stderr,
                    "ERROR: failed to create thread %d\n",
                    i);

            return EXIT_FAILURE;
        }
    }

    //wait for all threads
    for(int i = 0; i < num_threads; i++) {
        pthread_join(threads[i], NULL);
    }

    clock_gettime(CLOCK_MONOTONIC, &end);

    double elapsed = (end.tv_sec - start.tv_sec)
                   + (end.tv_nsec - start.tv_nsec) / 1e9;

    printf("Matrix size: %d x %d\n", N, N);
    printf("Threads: %d\n", num_threads);
    printf("Elapsed time: %.6f seconds\n", elapsed);

    //small correctness check
    printf("C[0][0]: %.6f\n", C[0]);

    free(A);
    free(B);
    free(C);

    return EXIT_SUCCESS;
}