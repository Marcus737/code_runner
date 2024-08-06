set -e

result_file="run_result.txt"
time_limit="$1"
memory_limit="$2"

cd /

cp code.src main.py


./watch_file -o "$result_file" -t "$time_limit" -m "$memory_limit" -- python3 main.py < input.txt