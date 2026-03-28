# To run this script, install vhs and run `vhs ./examples/tapes/variants.tape`
Output "examples/tapes/variants.gif"
Set Margin 10
Set Padding 2
Set BorderRadius 10
Set Width 1800
Set Height 800
Set PlaybackSpeed 1.0

Hide
Type "cargo run --example variants"
Enter
Sleep 1.5s
Show

# Classic
Sleep 1.0s
Right@0.2s 1
Down@0.1s 17

# Infinite
Left@0.5s 1
Down@0.5s 1

Right@0.2s 1
Down@0.15s 16
Sleep 1.0s

# Scroll padding
Left@0.5s 1
Down@0.5s 1

Right@0.2s 1
Down@0.15s 14
Up@0.15s 14

# Horizontal
Left@0.5s 1
Down@0.5s 1

Right@0.2s 1
Down@0.2s 5

# Reversed
Left@0.5s 1
Down@0.5s 1

Right@0.2s 1
Up@0.2s 6
Down@0.2s 5

Sleep 1.5s
