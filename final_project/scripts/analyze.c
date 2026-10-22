#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
#include <errno.h>

#define MAX_TRIALS 100
#define Z_SCORE 1.96
#define MAX_LABEL 64
#define MAX_PATH 512

typedef struct {
    char label[MAX_LABEL];
    double times[MAX_TRIALS];
    int n;
    double mean;
    double std;
    double moe;
    double moe_pct;
    double min;
    double max;
} Stats;

int load_csv(const char *path, Stats *s) {

    FILE *fp = fopen(path, "r");
    if(!fp) return -1;

    char line[256];
    int n = 0;

    //skip header
    if(!fgets(line, sizeof(line), fp)) {
        fclose(fp);
        return 0;
    }

    while(fgets(line, sizeof(line), fp) && n < MAX_TRIALS) {

        char *comma = strchr(line, ',');
        if(!comma) continue;

        double t = atof(comma + 1);
        if(t > 0.0) {
            s->times[n++] = t;
        }
    }

    fclose(fp);
    return n;
}

void compute_stats(Stats *s) {

    if(s->n < 1) return;

    double sum = 0.0;
    s->min = s->times[0];
    s->max = s->times[0];

    for(int i = 0; i < s->n; i++) {
        sum += s->times[i];

        if(s->times[i] < s->min) s->min = s->times[i];
        if(s->times[i] > s->max) s->max = s->times[i];
    }

    s->mean = sum / s->n;

    double var = 0.0;
    for(int i = 0; i < s->n; i++) {
        double d = s->times[i] - s->mean;
        var += d * d;
    }

    if(s->n > 1) {
        s->std = sqrt(var / (s->n - 1));
    } else {
        s->std = 0.0;
    }

    s->moe = Z_SCORE * (s->std / sqrt((double)s->n));

    if(s->mean > 0.0) {
        s->moe_pct = s->moe / s->mean * 100.0;
    } else {
        s->moe_pct = 0.0;
    }
}

void print_row(const Stats *s) {

    printf("  %-36s n=%2d  mean=%9.6fs  std=%9.6f  "
           "MoE=+/-%.6f (%5.1f%%)  [%.6f, %.6f]\n",
           s->label, s->n, s->mean, s->std,
           s->moe, s->moe_pct, s->min, s->max);
}