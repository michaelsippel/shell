#!/bin/sh

grep -Po '::\K.*$' $1 |
while read -r pattern
do
    if echo "$2" | grep -xq "${pattern}"; then
	grep -A 2 -Fx "::${pattern}" $1 | grep -m 1 -Po "$3: \K.*$" &&  echo "^-- in pattern ${pattern}" 1>&2 && exit
    else
	if [ $? -eq 2 ]; then
	   echo "^-- in pattern ${pattern}" 1>&2
	fi
    fi
done

