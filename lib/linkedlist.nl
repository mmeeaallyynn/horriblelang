"std.nl" include

blubb is

in


linked-list is
  element is value next in

  // initial value -> reference
  new is
    ref is _ in
    2 @memory::alloc! -> @::ref
    @::ref$ put -1 @::ref$ 1 + put
    @::ref$
  in

  next-item is
    dup -1 == not lambda
      1 + get
    in jump?
  in

  last-item is
    ref is _ in -> @::ref

    // find the end
    @::ref$ 1 + get -1 == not
    lambda
      @::ref$ 1 + get -> @::ref
      @::ref$ 1 + get -1 == not
      loop?
    in jump?
    @::ref$
  in

  // value ref
  push is
    ref is _ in
    @::last-item! -> @::ref

    // add element to the end
    @::new! @::ref$ 1 + put
  in

  // ref
  dump is
    ref is _ in -> @::ref

    [ print
    @::ref$$ print
    // find the end
    @::ref$ 1 + get -1 == not
    lambda
      @::ref$ 1 + get -> @::ref
      "," print \space print @::ref$$ print
      @::ref$ 1 + get -1 == not
      loop?
    in jump?
    ] print
    \n print
  in

  pop is
    // TODO: Handle list with only one item
    this is _ in -> @::this
    next is _ in
    @::this$ @::next-item! -> @::next

    @::next$ @::next-item! -1 == not
    lambda
      @::next$ -> @::this
      @::next$ @::next-item! -> @::next
      @::next$ 
      @::next-item! -1 == not
      loop?
    in jump?

    @::next$$
    @::next$ @memory::free!
    -1 @::this$ 1 + put
  in

  len is
    ref is _ in -> @::ref
    count is _ in 1 -> @count

    @::ref$ 1 + get -1 == not
    lambda
      @::count$ 1 + -> @::count
      @::ref$ 1 + get -> @::ref
      @::ref$ 1 + get -1 == not
      loop?
    in jump?

    @::count$
  in

  // takes index, ref
  at is
    ref is _ in -> @::ref

    0 swap lambda
      drop
      @::ref$ @::next-item! -> @::ref
    in @std::loop-range!

    @::ref$$
  in

  delete is
    // TODO
  in
in

