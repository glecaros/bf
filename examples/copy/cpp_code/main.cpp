#include <iostream>
#include "shared.h"

using namespace std;

int main(int argc, char *argv[])
{
  SomeClass m;

  cout << m.getX() << endl;
  m.setX(10);
  cout << m.getX() << endl;
}
