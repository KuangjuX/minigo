int a = 10;
long x = 100;
char* s = "123456";

int foo(int a, int b){
    return a + b;
}

int main() {
    int ret = foo(1, 2);
    ret = foo(a, ret);
    return ret;
}