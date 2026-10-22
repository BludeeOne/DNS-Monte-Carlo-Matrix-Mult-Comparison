/*
 * File: pager-predict.c
 * Author:       Erik Flores-Siemsen
 * Adopted From: Dr. Alva Couch
 *               http://www.cs.tufts.edu/~couch/
 *
 * Project: CSCI 440 paging assignment
 * Create Date: Unknown
 * Modify Date: 2026/04/07
 * Description:
 * 	This file contains a predictive pageit
 *      implmentation.
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
 
#include "simulator.h"
 
#define PG(pc)      ((pc) / PAGESIZE)
 
#define THRESH_P5   505
#define THRESH_P1  1131
#define THRESH_P2  1685
#define THRESH_P3  1912
 
#define KIND_UNKNOWN  0
#define KIND_P5       5
#define KIND_P1       1
#define KIND_P2       2
#define KIND_P3       3
#define KIND_P4       4
 
static void prefetch(int proc, int page) {
    if (page >= 0 && page < MAXPROCPAGES)
        pagein(proc, page);
}
 
void pageit(Pentry q[MAXPROCESSES]) {
 
    static int initialized = 0;
    static int tick = 1;
    static int timestamps[MAXPROCESSES][MAXPROCPAGES];
    static int kind[MAXPROCESSES];
    static int max_pc[MAXPROCESSES];
    static int stable[MAXPROCESSES];
 
    int proc, page, pc, cur_page, mpc;
    int ep, pg;
    int lru_proc, lru_page, lru_time;
 
    if (!initialized) {
        for (proc = 0; proc < MAXPROCESSES; proc++) {
            for (page = 0; page < MAXPROCPAGES; page++)
                timestamps[proc][page] = 0;
            kind[proc]   = KIND_UNKNOWN;
            max_pc[proc] = 0;
            stable[proc] = 0;
        }
        initialized = 1;
    }
 
    /* ---------------------------------------------------------------- */
    /* Pass 1: timestamps and classification                            */
    /* ---------------------------------------------------------------- */
    for (proc = 0; proc < MAXPROCESSES; proc++) {
        if (!q[proc].active) {
            kind[proc] = KIND_UNKNOWN; max_pc[proc] = 0; stable[proc] = 0;
            continue;
        }
        pc   = (int)q[proc].pc;
        page = PG(pc);
        if (q[proc].pages[page]) timestamps[proc][page] = tick;
        if (pc > max_pc[proc])   max_pc[proc] = pc;
        if (!stable[proc]) {
            mpc = max_pc[proc];
            if      (mpc < THRESH_P5) kind[proc] = KIND_P5;
            else if (mpc < THRESH_P1) kind[proc] = KIND_P1;
            else if (mpc < THRESH_P2) kind[proc] = KIND_P2;
            else if (mpc < THRESH_P3) kind[proc] = KIND_P3;
            else                       kind[proc] = KIND_P4;
 
            if      (kind[proc]==KIND_P5 && mpc>400)  stable[proc]=1;
            else if (kind[proc]==KIND_P1 && mpc>900)  stable[proc]=1;
            else if (kind[proc]==KIND_P2 && mpc>1000) stable[proc]=1;
            else if (kind[proc]==KIND_P3 && mpc>1200) stable[proc]=1;
            else if (kind[proc]==KIND_P4 && mpc>1800) stable[proc]=1;
        }
    }
 
    /* ---------------------------------------------------------------- */
    /* Pass 2: proactively evict all P4 pages except current and next   */
    /* ---------------------------------------------------------------- */
    for (proc = 0; proc < MAXPROCESSES; proc++) {
        if (!q[proc].active || kind[proc] != KIND_P4) continue;
        cur_page = PG((int)q[proc].pc);
        for (pg = 0; pg < MAXPROCPAGES; pg++) {
            if (pg == cur_page || pg == cur_page + 1) continue;
            if (q[proc].pages[pg]) pageout(proc, pg);
        }
    }
 
    /* ---------------------------------------------------------------- */
    /* Pass 3: paging decisions                                         */
    /* ---------------------------------------------------------------- */
    for (proc = 0; proc < MAXPROCESSES; proc++) {
        if (!q[proc].active) continue;
 
        pc       = (int)q[proc].pc;
        cur_page = PG(pc);
 
        /* Step A: ensure current page is in; evict LRU if needed */
        if (!pagein(proc, cur_page)) {
            lru_proc = -1; lru_page = -1; lru_time = tick + 1;
            for (ep = 0; ep < MAXPROCESSES; ep++) {
                if (!q[ep].active) continue;
                for (pg = 0; pg < MAXPROCPAGES; pg++) {
                    if (!q[ep].pages[pg]) continue;
                    if (pg == PG((int)q[ep].pc)) continue;
                    if (timestamps[ep][pg] == 0) {
                        lru_proc = ep; lru_page = pg;
                        ep = MAXPROCESSES; break;
                    }
                    if (timestamps[ep][pg] < lru_time) {
                        lru_time = timestamps[ep][pg];
                        lru_proc = ep; lru_page = pg;
                    }
                }
            }
            if (lru_proc != -1) pageout(lru_proc, lru_page);
            continue;
        }
 
        /* Step B: prefetch next sequential page */
        prefetch(proc, cur_page + 1);
 
        /* Step C: prefetch known jump targets */
        switch (kind[proc]) {
        case KIND_P5:
            prefetch(proc, 0); prefetch(proc, 1);
            prefetch(proc, 2); prefetch(proc, 3);
            break;
        case KIND_P1:
            prefetch(proc, 0);
            if (cur_page == 3) prefetch(proc, 10);
            break;
        case KIND_P2:
            if (cur_page >= 6) prefetch(proc, 0);
            break;
        case KIND_P3:
            if (cur_page >= 11) { prefetch(proc, 9); prefetch(proc, 0); }
            if (cur_page == 9)  prefetch(proc, 13);
            if (cur_page == 0)  prefetch(proc, 9);
            break;
        case KIND_P4:
            break;
        case KIND_UNKNOWN:
        default:
            prefetch(proc, 0);
            break;
        }
    }
 
    tick++;
}