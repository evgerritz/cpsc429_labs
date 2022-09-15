cmd_/home/evgerritz/mymem/Module.symvers := sed 's/\.ko$$/\.o/' /home/evgerritz/mymem/modules.order | scripts/mod/modpost -m -a  -o /home/evgerritz/mymem/Module.symvers -e -i Module.symvers   -T -
