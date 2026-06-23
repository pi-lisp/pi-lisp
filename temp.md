i tried test the cubical with [test.uwuc](file;file:///home/jihoo/uwulisp/test.uwuc) using "cargo run -- --cubical test.uwuc" command and i got this error
Checking definition: id
Checking definition: plus
Checking definition: two
Checking definition: four
Checking definition: isZero
Checking definition: loopPath
Checking definition: flipLoop
Checking definition: swap
Checking definition: compose
Checking definition: symPath
Checking definition: both
Checking definition: cast
Checking definition: transportExample
Cubical error: type error:
  Type mismatch
    expected : #2
    got      : #3
and it's parser and other backends are self contained in [cubical](file;file:///home/jihoo/uwulisp/src/cubical) thus you don't need see other directory 
can you fix the error?