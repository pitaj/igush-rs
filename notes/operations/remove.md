## remove(1)

remove first element in last row
```
    0 1 2 3
0 [ c d|a b ]
1 [ h|e f g ]
2 [ j k l|i ]
3 [|m n     ]
    R
```

swap into previous row
```
    0 1 2 3
0 [ c d|a b ]
1 [ h|e f g ]
2 [ j k l|i ] m
          ^   ^
3 [|n       ]
```

move split
```
    0 1 2 3
0 [ c d|a b ]
1 [ h|e f g ] i
2 [ j k l|m ]
         >>|
3 [|n       ]
```

swap into previous row
```
    0 1 2 3
0 [ c d|a b ]
1 [ h|e f g ] i
      ^       ^
2 [ j k l m|]
3 [|n       ]
```

move split
```
    0 1 2 3
0 [ c d|a b ] e
1 [ h|i f g ]
     >>|
2 [ j k l m|]
3 [|n       ]
```

swap into target row
```
    0 1 2 3
0 [ c d|a b ] e
          ^   ^
1 [ h i|f g ]
2 [ j k l m|]
3 [|n       ]
```

rotate into position
```
    0 1 2 3
0 [ c d|a e ]
       {   } rotate_left(1)
1 [ h i|f g ]
2 [ j k l m|]
3 [|n       ]

b
```

move split
```
    0 1 2 3
0 [ c d|e a ]
       >>|
1 [ h i|f g ]
2 [ j k l m|]
3 [|n       ]

b
```

done
```
    0 1 2 3
0 [ c d e|a ]
1 [ h i|f g ]
2 [ j k l m|]
3 [|n       ]

b
```
