i tried test the cubical with [test.uwuc](file;file:///home/jihoo/uwulisp/test.uwuc) using "cargo run -- --cubical test.uwuc" command and i got this error
Cubical error: type error:
  endpoint mismatch (ctx_depth=9, ctx=["i", "x", "flipLoop", "loopPath", "isZero", "four", "two", "plus", "id"])
  expected=Path S1 base base  [raw=Path S1 base base]
  got=(loop @ ¬0)  [raw=(loop @ ¬0)]
and it's parser and other backends are self contained in [cubical](file;file:///home/jihoo/uwulisp/src/cubical) thus you don't need see other directory 
can you fix the error?