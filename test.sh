[ -d build ] || premake gmake
cd build
make test && ./test/test_program

