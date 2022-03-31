memory {
  mem is _1000 in
  idx is 0 in
  free-list is _100 in
  free-list-len is 50 in
  block-list is _100 in
  block-list-len is 50 in

  init-free-list is
    0 @memory::free-list-len$ @lists::range!
    lambda
      idx is _ in
      2 * -> @idx
      -1 @memory::free-list @idx$ + put
      -1 @memory::free-list @idx$ + 1 + put
    in @lists::foreach!
  in

  init-block-list is
    0 @memory::block-list-len$ @lists::range!
    lambda
      idx is _ in
      2 * -> @idx
      -1 @memory::block-list @idx$ + put
      -1 @memory::block-list @idx$ + 1 + put
    in @lists::foreach!
  in

  // takes: position size
  add-free-block is
    size is _ in
    position is _ in
    idx is _ in
    -> @size
    -> @position
    0 -> @idx

    lambda
      @memory::free-list @idx$ + 1 + get
      (@idx$ + 2) -> @idx
      -1 != loop?
    in jump

    @size$ @memory::free-list @idx$ 1 - + put
    @position$ @memory::free-list @idx$ 2 - + put
  in

  // takes: position size
  add-block is
    size is _ in
    position is _ in
    idx is _ in
    -> @size
    -> @position
    0 -> @idx

    lambda
      @memory::block-list @idx$ + 1 + get
      (@idx$ + 2) -> @idx
      -1 != loop?
    in jump

    @size$ @memory::block-list @idx$ 1 - + put
    @position$ @memory::block-list @idx$ 2 - + put
  in


  // try to find a block to fuse with
  try-fuse-free-blocks is
    check-fusable is
      
    in
    
  in

  init is
    @memory::init-free-list!
    @memory::init-block-list!
    0 1000 @memory::add-free-block!
  in

  alloc is
    idx is _ in
    size is _ in
    -> @size
    0 -> @idx

    // find space in the free list
    lambda
      @memory::free-list @idx$ + 1 + get
      (@idx$ + 2) -> @idx
      @size$ <= loop?
    in jump

    // TODO memory full?

    size-ref is
        @memory::free-list swap 1 - +
    in
    position-ref is
        @memory::free-list swap 2 - +
    in

    // add entry to the block list
    @idx$ @position-ref!$
    @size$ @memory::add-block!

    // update free-list entry
    @memory::alloc::idx$ @size-ref!$ @size$ - @memory::alloc::idx$ @size-ref! put
    @memory::mem @memory::alloc::idx$ @position-ref!$ +
    @memory::alloc::idx$ @position-ref!$ @size$ + @memory::alloc::idx$ @position-ref! put
    // TODO remove free-block if size == 0
  in

  free is
    mem-offset is _ in
    // find offset in mem
    @memory::mem - -> @mem-offset

    // find size of the block
    idx is _ in
    0 -> @idx
    lambda
      @memory::block-list @idx$ + get
      (@idx$ + 2) -> @idx
      @mem-offset$ != loop?
    in jump

    // add block to free list
    @mem-offset$ @memory::block-list @idx$ 1 - + get @memory::add-free-block!

    // remove block entry from block list
    -1 @memory::block-list @memory::free::idx$ 1 - + put
    -1 @memory::block-list @memory::free::idx$ 2 - + put
  in

  dump-free-list is
    idx is _ in
    0 -> @::idx
    lambda
      @::free-list @::idx$ + get -1 != lambda
        position: print \space print
        @::free-list @::idx$ + get print
        \space print
        size: print \space print
        @::free-list @::idx$ + 1 + get print
        \n print
      in jump?
      (@::idx$ + 2) -> @idx
      @::idx$ @::free-list-len$ 2 * < loop?
    in jump
  in

  dump-block-list is
    idx is _ in
    0 -> @::idx
    lambda
      @::block-list @::idx$ + get -1 != lambda
        position: print \space print
        @::block-list @::idx$ + get print
        \space print
        size: print \space print
        @::block-list @::idx$ + 1 + get print
        \n print
      in jump?
      (@::idx$ + 2) -> @idx
      @::idx$ @::block-list-len$ 2 * < loop?
    in jump
  in
}

@memory::init!

// functions for working with function local variables
local {
  stack is _100 in
  stack-idx is -1 in

  enter is
    @local::stack-idx @std::inc!
    dup @local::stack @local::stack-idx$ + put
    dup 2 + @std::inc!
    jump
  in
  exit is
    @local::stack @local::stack-idx$ + get 2 + @std::dec! 
    @local::stack-idx @std::dec!
  in
  access is @local::stack @local::stack-idx$ + get 2 + get + in
  "get" is @local::access! get in
  "put" is @local::access! put in
}
