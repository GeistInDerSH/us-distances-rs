# US Distances RS

A Rust implementation of the Clojure US Distances program.

This offers several optimizations over the Clojure version, that allows it
to be faster:

* 32-bit floats over 64-bit floats
* Work-stealing threading model