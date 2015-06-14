#!/bin/sh

DIR=./src/parser

[ "$DIR/grammar.rustpeg" -nt "$DIR/grammar.rs" ] && {
  echo "Updating grammar.rs"
  peg "$DIR/grammar.rustpeg" > "$DIR/grammar.rs"
}

[ "$DIR/complete.rustpeg" -nt "$DIR/complete.rs" ] && {
  echo "Updating complete.rs"
  peg "$DIR/complete.rustpeg" > "$DIR/complete.rs"
}
