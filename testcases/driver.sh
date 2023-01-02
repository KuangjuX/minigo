minigo=$1


tmp=`mktemp -d /tmp/chibicc-test-XXXXXX`
trap 'rm -rf $tmp' INT TERM HUP EXIT
echo > $tmp/empty.c

check() {
    if [ $? -eq 0 ]; then
        echo "testing $1 ... passed"
    else
        echo "testing $1 ... failed"
        exit 1
    fi
}



qemu-riscv64  testcases/add.exe
[ "$?" = 15 ]
check add

qemu-riscv64  testcases/sub.exe
[ "$?" = 6 ]
check sub

qemu-riscv64  testcases/mul.exe
[ "$?" = 56 ]
check mul  

qemu-riscv64  testcases/div.exe
[ "$?" = 7 ]
check div

qemu-riscv64  testcases/icmp.exe
[ "$?" = 1 ]
check icmp

qemu-riscv64  testcases/if.exe
[ "$?" = 1 ]
check if

qemu-riscv64  testcases/while.exe
[ "$?" = 0 ]
check while

qemu-riscv64  testcases/call.exe
[ "$?" = 42 ]
check call

qemu-riscv64  testcases/add_many_regs.exe
[ "$?" = 134 ]
check add_many_regs


