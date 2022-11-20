#! /bin/bash

# kill also the background function on CRTL-C
trap 'trap - SIGTERM && kill 0' SIGINT SIGTERM EXIT

sleep_time=0.4
term_cols=$(tput cols)
width=$((term_cols / 2))
echo the width is: $width

query_contracts_forever() {
    while :
    do
        local result_1
        local result_2

        # query chain 1 and save output to file
        result_1=$(secretcli q compute query $(head -n 1 ./contract-addresses.log) '{"last_ibc_operation":{}}' --node 'tcp://localhost:26657' 2>&1 | fold -w $width)
        #echo "got result: $result_1"
        echo -e "chain 1\n$result_1" > output-query-1.log

        # query chain 2 and save output to a second file
        result_2=$(secretcli q compute query $(tail -n 1 ./contract-addresses.log) '{"last_ibc_operation":{}}' --node 'tcp://localhost:36657' 2>&1 | fold -w $width)
        # echo "got result: $result_2"
        echo -e "chain 2\n$result_2" > output-query-2.log

        sleep $sleep_time
    done
}

#query_contracts_forever &
query_contracts_forever &

# display the queries' output side by side
watch -n $sleep_time "pr -m -t -w 90 output-query-1.log output-query-2.log"
