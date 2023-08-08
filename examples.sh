
# arithmetic on timestamp
date +%s | xargs expr 2 +

# arithmetic on file count
ls -a | wc -l | xargs expr 2 +

## Count to 20 in hexadecimal
seq 20 | xargs -I@ -n 1 dc -e "16 o 10 i @ p"

## Try to mix radices in arithmetic
seq 10 | xargs -I@ -n 1 dc -e "8 o 10 i @ p" | xargs -n 1 expr 2 +

## Show paths of top 3 most recently modified files
find
| xargs -n 1 stat -c %Y,%n
| sort -rn
| head -n 3
| cut -d, -f2

