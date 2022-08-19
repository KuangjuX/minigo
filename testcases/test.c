int a = 10;
int b;


struct S {
    int x;
    char c;
}; 

struct S m;

int foo(int a, int b){
    return a + b;
}

int main() {
    struct S s;
    s.x = 1;
    int ret = foo(1, 2);
    return ret;
}