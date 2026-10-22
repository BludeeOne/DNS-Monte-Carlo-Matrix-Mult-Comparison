/*
 * File: pager-lru.c
 * Author:       Erik FLores-Siemsen
] * Adopted From: Dr. Alva Couch
 *               http://www.cs.tufts.edu/~couch/
 *
 * Project: CSCI 440 paging
 * Create Date: Unknown
 * Modify Date: 2026/05/07
 * Description:
 * 	This file contains an lru pageit
 *      implmentation.
 */

#include <stdio.h>
#include <stdlib.h>
 
#include "simulator.h"
 
void pageit(Pentry q[MAXPROCESSES]) {
 
    static int initialized = 0;
    static int tick = 1;
    static int timestamps[MAXPROCESSES][MAXPROCPAGES];
 
    int proc, page, need, ep, pg;
    int lru_proc, lru_page, lru_time;
 
    if (!initialized) {
        for (proc = 0; proc < MAXPROCESSES; proc++)
            for (page = 0; page < MAXPROCPAGES; page++)
                timestamps[proc][page] = 0;
        initialized = 1;
    }
 
    /* Step 1: stamp currently executing page for each active process */
    for (proc = 0; proc < MAXPROCESSES; proc++) {
        if (!q[proc].active) continue;
        need = q[proc].pc / PAGESIZE;
        if (q[proc].pages[need])
            timestamps[proc][need] = tick;
    }
 
    /* Step 2: ensure each active process has its current page in */
    for (proc = 0; proc < MAXPROCESSES; proc++) {
        if (!q[proc].active) continue;
 
        need = q[proc].pc / PAGESIZE;
        if (q[proc].pages[need]) continue;
        if (pagein(proc, need)) continue;
 
        /* pagein failed — find LRU page that its owner doesn't currently need */
        lru_proc = -1;
        lru_page = -1;
        lru_time = tick + 1;
 
        for (ep = 0; ep < MAXPROCESSES; ep++) {
            if (!q[ep].active) continue;
            for (pg = 0; pg < MAXPROCPAGES; pg++) {
                if (!q[ep].pages[pg]) continue;
 
                /* Don't evict the page ep's PC is currently on */
                if (pg == (int)(q[ep].pc / PAGESIZE)) continue;
 
                if (timestamps[ep][pg] == 0) {
                    lru_proc = ep;
                    lru_page = pg;
                    ep = MAXPROCESSES; /* break outer loop */
                    break;
                }
                if (timestamps[ep][pg] < lru_time) {
                    lru_time = timestamps[ep][pg];
                    lru_proc = ep;
                    lru_page = pg;
                }
            }
        }
 
        if (lru_proc != -1)
            pageout(lru_proc, lru_page);
    }
 
    tick++;
}