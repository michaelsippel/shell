cd src

find . -name *.rs | xargs -n 1 stat -c %X,%n | sort -rn | cut -d, -f2 | head -n 3

ls -a | wc -l | xargs expr 2 +

date +%s | xargs expr 2 +

