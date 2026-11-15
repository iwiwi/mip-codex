/* Comparison-only GLPK driver, GPL-3.0-or-later (links libglpk).
 * This executable is never invoked by the Rust solver.
 */
#include <glpk.h>
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
typedef struct { double bound; int nodes; int direction; } Capture;
static void callback(glp_tree *tree, void *data) {
 Capture *c=data;int active,current,total;glp_ios_tree_size(tree,&active,&current,&total);c->nodes=total;
 int node=glp_ios_best_node(tree);
 if(node){double b=glp_ios_node_bound(tree,node);if(isfinite(b)&&fabs(b)<1e100){if(!isfinite(c->bound)||(c->direction==GLP_MIN?b>c->bound:b<c->bound))c->bound=b;}}
}
static void number(FILE *f,double x){if(isfinite(x)&&fabs(x)<1e100)fprintf(f,"%.17g",x);else fputs("null",f);}
static void string(FILE *f,const char *s){fputc('"',f);for(const unsigned char*p=(const unsigned char*)(s?s:"");*p;p++){if(*p=='"'||*p=='\\'){fputc('\\',f);fputc(*p,f);}else if(*p<32)fprintf(f,"\\u%04x",*p);else fputc(*p,f);}fputc('"',f);}
int main(int argc,char **argv){
 if(argc!=4){fprintf(stderr,"usage: glpk_driver input.mps seconds output.json\n");return 2;}
 glp_term_out(GLP_OFF);glp_prob *p=glp_create_prob();if(glp_read_mps(p,GLP_MPS_FILE,NULL,argv[1]))return 3;
 glp_iocp options;glp_init_iocp(&options);options.presolve=GLP_ON;options.tm_lim=(int)(atof(argv[2])*1000);options.mip_gap=1e-6;
 Capture capture={NAN,0,glp_get_obj_dir(p)};options.cb_func=callback;options.cb_info=&capture;
 int code=glp_intopt(p,&options);int status=glp_mip_status(p);int feasible=status==GLP_OPT||status==GLP_FEAS;
 FILE *f=fopen(argv[3],"w");if(!f)return 4;
 fprintf(f,"{\"status\":\"%s\",\"returncode\":%d,\"nodes\":%d,\"objective\":",status==GLP_OPT?"optimal":status==GLP_FEAS?"feasible_limit":status==GLP_NOFEAS?"infeasible":code==GLP_ETMLIM?"time_limit":"undefined",code,capture.nodes);
 number(f,feasible?glp_mip_obj_val(p):NAN);fputs(",\"dual_bound\":",f);number(f,status==GLP_OPT?glp_mip_obj_val(p):capture.bound);
 fputs(",\"values\":",f);int n=glp_get_num_cols(p);if(feasible){fputc('[',f);for(int j=1;j<=n;j++){if(j>1)fputc(',',f);number(f,glp_mip_col_val(p,j));}fputc(']',f);}else fputs("null",f);
 fputs(",\"variable_names\":[",f);for(int j=1;j<=n;j++){if(j>1)fputc(',',f);string(f,glp_get_col_name(p,j));}fputs("]}\n",f);fclose(f);glp_delete_prob(p);return 0;
}
