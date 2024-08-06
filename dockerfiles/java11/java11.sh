#!/usr/bin/env sh

result_file="run_result.txt"
time_limit="$1"
memory_limit="$2"

write_error_result(){
  echo "status=$1" > $result_file
}

check() {
  res="$?";
  if [ $res -ne 0 ]; then
      write_error_result "$1"
      exit 1
  fi
}

cd /

cp code.src Main.java
check "script error"

javac Main.java
check "compile error"

./watch_file -o "$result_file" -t "$time_limit" -- java -Xmx"$memory_limit"m Main < input.txt
check "program error"
