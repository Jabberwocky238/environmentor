
#include <iostream>
using namespace std;

class A
{
    int a;

public:
    A(int i = 0) : a(i) { 
        cout << a << " A constructor\n"; 
    }
    ~A() { cout << a << " A distructor\n"; }
};
class B : public A
{
    A a;
    int b;
public:
    B(int i = 1, int j = 2) : a(i), b(j) { 
        cout << b << " B constructor\n"; 
    }
    ~B() { cout << b << " B distructor\n"; }
};

int main() {
    B ob1;
    A ob2;
    return 0;
}