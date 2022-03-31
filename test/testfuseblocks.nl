"testll.nl" include


"free list" @std::println!
@memory::dump-free-list!


// takes base, element size, idx
get-memory-at is
  idx is _ in -> @::idx
  size is _ in -> @::size
  base is _ in -> @::base

  @base$ (@idx$ * @size$) +
in

exists is
  dup -1 !=
in

@memory::free-list 2 4 @get-memory-at!

try-fuse is
  a-position is _ in -> @::a-position
  a-size is _ in -> @::a-size
  end is _ in @::a-position$ @::a-size$ + -> @::end

  b-position is _ in
  b-size is _ in

  lambda
    

    loop?
  in jump
in


dup 1 + get
swap get
@try-fuse!

STACK
