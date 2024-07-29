#!/usr/bin/env sh

result_file="result.txt"
time_limit="$1"
memory_limit="$2"

write_error_result(){
  echo "status=$1" > $result_file
}

write_ok_result(){
  echo "status=ok" >> $result_file
}

check() {
  res="$?";
  if [ $res -eq 124 ]; then
      write_error_result "timeout"
      exit 2
  fi
  if [ $res -ne 0 ]; then
      write_error_result "$1"
      exit 1
  fi
}

cp code.src Main.java
check "script error"

javac Main.java
check "compile error"

/bin/time -q -f "use_time=%e\nmax_mem=%M\n" -o $result_file timeout --foreground "$time_limit"s java -Xmx"$memory_limit"m Main < input.txt
check "program error"

write_ok_result