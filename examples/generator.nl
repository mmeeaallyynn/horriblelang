
generator is
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
  func is _ in
  value is 0 in
  ref is _ in
  // called to generate the next value
  next is
    @generator::value! dup @generator::func!!
    -> @generator::value
  in
in


gena is _ in
genb is _ in

4 @memory::alloc! lambda 1 + in 0 @generator::new! -> @gena
4 @memory::alloc! lambda 2 * in 1 @generator::new! -> @genb

"generator a" print
@gena! @generator::load!
@generator::next! print
@generator::next! print
@generator::save!

"generator b" print
@genb! @generator::load!
@generator::next! print
@generator::next! print
@generator::next! print
@generator::next! print

"generator a" print
@gena! @generator::load!
@generator::next! print
@generator::next! print

