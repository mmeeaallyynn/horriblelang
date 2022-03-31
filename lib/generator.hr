
generator is
  // consumes initial initial value, function point, reference to memory with 3 spaces 
  // produces reference to memory
  // write params for new generator a specified memory location
  new is
    -> @generator::value
    -> @generator::func
    -> @generator::ref 
    "gen" @generator::ref! 2 + put
    @generator::value! @generator::ref! 1 + put
    @generator::func! @generator::ref! put
    @generator::ref!
  in
  // load a generator from an address
  load is
    -> @generator::ref 
    @generator::ref!!
    "gen" != dup lambda "not a generator!" print in jump?
    not lambda
      -> @generator::value
      -> @generator::func
    in jump?
  in
  // save the state of the current generator
  save is
    @generator::ref!
    @generator::func!
    @generator::value!
    @generator::new!
    drop
  in
  // consumes integer number
  // produce the next n items as a list
  take is
    @lists::guard! swap 
    loop {
      1 - dup
      0 >= lambda
        @generator::next!
        swap
        @loop!
      in jump?
    } @loop! 

    drop
  in

  func is _ in
  value is 0 in
  ref is _ in
  // called to generate the next value
  next is
    @generator::value! dup @generator::func!!
    -> @generator::value
  in
in

