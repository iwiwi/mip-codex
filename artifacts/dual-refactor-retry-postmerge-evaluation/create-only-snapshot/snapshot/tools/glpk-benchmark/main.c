/* SPDX-License-Identifier: GPL-3.0-or-later
 * Copyright (C) 2026 mip-codex contributors
 * Comparison-only GLPK driver. Never linked into the Rust solver.
 * GLPK default MIP parameters, with cuts/FP enabled as in the official
 * MIPLIB2017 run_glpk.sh. Callback observes bounds; it changes no decisions.
 */
#include <glpk.h>
#include <float.h>
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
static double last_bound=NAN;
static int created=0;
static double last_poll=-1e100,last_bound_time=NAN;
static double wall(void) {
    struct timespec t;clock_gettime(CLOCK_MONOTONIC,&t);return t.tv_sec+1e-9*t.tv_nsec;
}
static void observe(glp_tree *tree,void *unused) {
    (void)unused;
    int a,n,t;glp_ios_tree_size(tree,&a,&n,&t);created=t;
    double now=wall();if(now-last_poll<1.0)return;last_poll=now;
    int best=glp_ios_best_node(tree);
    if(best) {double b=glp_ios_node_bound(tree,best);if(isfinite(b)&&fabs(b)<DBL_MAX){last_bound=b;last_bound_time=now;}}
}
static void number(FILE *f,double x) {if(!isfinite(x))fputs("null",f);else if(fabs(x)>=9007199254740992.0)fprintf(f,"%.0f",x);else fprintf(f,"%.17g",x);}
int main(int argc,char **argv) {
    if(argc!=5) {fprintf(stderr,"usage: glpk-benchmark INPUT SECONDS OUTPUTDIR MEMORY_MIB\n");return 2;}
    double seconds=strtod(argv[2],NULL);
    if(!isfinite(seconds)||seconds<0||seconds>604800)return 2;
    glp_mem_limit(atoi(argv[4]));
    glp_prob *p=glp_create_prob();
    if(glp_read_mps(p,GLP_MPS_FILE,NULL,argv[1]))return 3;
    /* CPLEX/HiGHS convention used by our original-MPS checker: the
       objective offset is the NEGATIVE RHS of the objective row.
       GLPK's MPS extension reads it with the opposite sign. */
    double old_offset=glp_get_obj_coef(p,0);glp_set_obj_coef(p,0,-old_offset);
    char path[8192];snprintf(path,sizeof(path),"%s/model.glp",argv[3]);
    if(glp_write_prob(p,0,path))return 4;
    int status,ret;double primal=NAN,elapsed;
    double started=wall();
    glp_iocp ip;glp_init_iocp(&ip);
    glp_smcp sp;glp_init_smcp(&sp);
    ip.presolve=GLP_ON;ip.tm_lim=(int)(seconds*1000);ip.mip_gap=0;
    ip.gmi_cuts=ip.mir_cuts=ip.cov_cuts=ip.clq_cuts=GLP_ON;ip.fp_heur=GLP_ON;
    ip.cb_func=observe;
    if(glp_get_num_int(p)>0) {
        ret=glp_intopt(p,&ip);elapsed=wall()-started;status=glp_mip_status(p);
        if(status==GLP_OPT||status==GLP_FEAS)primal=glp_mip_obj_val(p);
        if(status==GLP_OPT)last_bound=primal;
        snprintf(path,sizeof(path),"%s/solution.raw",argv[3]);glp_write_mip(p,path);
    } else {
        sp.presolve=GLP_ON;sp.tm_lim=(int)(seconds*1000);
        ret=glp_simplex(p,&sp);elapsed=wall()-started;status=glp_get_status(p);
        if(status==GLP_OPT||status==GLP_FEAS)primal=glp_get_obj_val(p);
        if(status==GLP_OPT)last_bound=primal;
        snprintf(path,sizeof(path),"%s/solution.raw",argv[3]);glp_write_sol(p,path);
    }
    if(status==GLP_OPT||status==GLP_FEAS) {
        snprintf(path,sizeof(path),"%s/values-api.tsv",argv[3]);FILE *values=fopen(path,"w");if(!values)return 5;
        int mip=glp_get_num_int(p)>0;
        for(int j=1;j<=glp_get_num_cols(p);++j) {
            fprintf(values,"%d ",j);number(values,mip?glp_mip_col_val(p,j):glp_get_col_prim(p,j));fputc('\n',values);
        }
        fclose(values);
    }
    snprintf(path,sizeof(path),"%s/native.json",argv[3]);FILE *f=fopen(path,"w");if(!f)return 5;
    const char *s=status==GLP_OPT?"optimal":status==GLP_FEAS?"feasible_limit":status==GLP_NOFEAS?"infeasible":status==GLP_UNBND?"unbounded":"unknown";
    fprintf(f,"{\"version\":\"%s\",\"status\":\"%s\",\"return_code\":%d,\"objective\":",glp_version(),s,ret);number(f,primal);
    fputs(",\"dual_bound\":",f);number(f,last_bound);
    fputs(",\"dual_bound_age_seconds\":",f);number(f,status==GLP_OPT?0:wall()-last_bound_time);
    fputs(",\"objective_offset_before\":",f);number(f,old_offset);
    fputs(",\"objective_offset_after\":",f);number(f,-old_offset);
    fprintf(f,",\"seconds\":%.17g,\"nodes_created\":%d,\"integer_tolerance\":%.17g,\"objective_tolerance\":%.17g}\n",elapsed,created,ip.tol_int,ip.tol_obj);
    fclose(f);glp_delete_prob(p);glp_free_env();return 0;
}
