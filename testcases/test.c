int a = 10;
int b;
static int d = 20;
long x = 100;
char* s = "123456";


const int e = 10;

int foo(int a, int b){
    return a + b;
}

int main() {
    int ret = foo(1, 2);
    ret = foo(a, ret);
    return ret;
}