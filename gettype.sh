#!/bin/sh

grep -Po '::\K.*$' $1 |
while read -r pattern
do
    if echo "$2" | grep -q "^${pattern}$"; then
	grep -A 2 -F "::${pattern}" $1 | grep -m 1 -Po "$3: \K.*$" && exit
    else
	if [ $? == 2 ]; then
	   echo "^-- in pattern ${pattern}"
	fi
    fi
done

