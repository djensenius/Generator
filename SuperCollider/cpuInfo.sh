#!/bin/bash
uptime | sed 's/ //'
echo "Host: $(hostname) $(hostname -I)"
vcgencmd measure_temp | sed "s/temp=/Temprature: /" | sed "s/'/°/"
vcgencmd measure_volts | sed "s/volt=/Power: /" | sed "s/v/ volts/"
