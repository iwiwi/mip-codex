#include <stdio.h>
#include <glpk.h>
int main(int argc,char **argv) {
 if(argc!=3)return 2;
 glp_term_out(GLP_OFF);
 glp_prob *p=glp_create_prob();
 if(glp_read_mps(p,GLP_MPS_FILE,NULL,argv[1]) || glp_read_mip(p,argv[2]))return 3;
 double ae,re;int ai,ri;
 glp_check_kkt(p,GLP_MIP,GLP_KKT_PE,&ae,&ai,&re,&ri);
 printf("row_equalities abs_error %.17g index %d relative %.17g\n",ae,ai,re);
 glp_check_kkt(p,GLP_MIP,GLP_KKT_PB,&ae,&ai,&re,&ri);
 printf("bounds abs_error %.17g index %d relative %.17g\n",ae,ai,re);
 printf("row5862 lower %.17g upper %.17g value %.17g\n",glp_get_row_lb(p,5862),glp_get_row_ub(p,5862),glp_mip_row_val(p,5862));
 glp_delete_prob(p);return 0;
}
