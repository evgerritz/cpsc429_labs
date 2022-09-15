cmd_/home/evgerritz/mymem/modules.order := {   echo /home/evgerritz/mymem/mymem.ko; :; } | awk '!x[$$0]++' - > /home/evgerritz/mymem/modules.order
