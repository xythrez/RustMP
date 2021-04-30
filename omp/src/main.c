#include <stdio.h>
#include <stdlib.h>
#include <sys/sysinfo.h>
#include <time.h>
#include <limits.h>
#include <unistd.h>

void
print_time_diff (struct timespec *start, struct timespec *end)
{
  time_t sec;
  long nsec;
  sec = end->tv_sec - start->tv_sec;
  nsec = end->tv_nsec - start->tv_nsec;

  if (nsec < 0)
    {
      sec--;
      nsec += 1000000000;
    }

  printf ("Elapsed time: %ld.%09ld sec\n", sec, nsec);
}

void
generate_matrices (double ***matrix, double ***result, size_t nsize)
{
    size_t i, j;
    *matrix = (double **) malloc (nsize * sizeof (double *));
    *result = (double **) malloc (nsize * sizeof (double *));
    srand (time (NULL));
    for (i = 0; i < nsize; i++) {
        (*matrix)[i] = (double *) malloc (nsize * sizeof (double));
        (*result)[i] = (double *) malloc (nsize * sizeof (double));
        for (j = 0; j < nsize; j++) {
            (*matrix)[i][j] = 255.0f * ((double) rand () - ((double) RAND_MAX/2.0f));
            (*result)[i][j] = 0;
        }
    }
}

void
free_all (double **matrix, double **result, size_t nsize)
{
    size_t i;
    for (i = 0; i < nsize; i++) {
        free (matrix[nsize]);
        free (result[nsize]);
    }
    free (matrix);
    free (result);
}


int main (int argc, char *argv[])
{
    struct timespec start, end;
    size_t i, j, k, nsize;
    long raw_size;
    double sum;
    double **matrix, **result;
    if (argc != 2) {
        printf ("Usage: %s <nsize>\n", argv[0]);
        exit(-1);
    }
    raw_size = atol (argv[1]);
    if (raw_size < 1) {
        printf ("Usage: %s <nsize>\n", argv[0]);
        exit(-1);
    }
    nsize = raw_size;
    generate_matrices (&matrix, &result, nsize);
    clock_gettime (CLOCK_MONOTONIC, &start);
#ifdef PAR
    #pragma omp parallel for default(shared) private(j, k, sum)
#endif
    for (i = 0; i < nsize; i++) {
        for (j = 0; j < nsize; j++) {
            sum = 0;
            for (k = 0; k < nsize; k++) {
                sum += matrix[i][k] * matrix[k][j];
            }
#ifdef PAR
#ifdef CRIT
            #pragma omp critical
            {
#endif
#endif
                result[i][j] = sum;
#ifdef PAR
#ifdef CRIT
            }
#endif
#endif
        }
    }
    clock_gettime (CLOCK_MONOTONIC, &end);
    print_time_diff (&start, &end);
    free_all (matrix, result, nsize);
}
