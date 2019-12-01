## insert(4, x)

swap in new element
with last element
```
    0 1 2 3
0 [ c d|a b ]
1 [ h|e f g ] x
    ^         ^
2 [ j k l|i ]
3 [|m n     ]
```

rotate into position
```
    0 1 2 3
0 [ c d|a b ]
1 [ x|e f g ]
   {     } rotate_left(1)
2 [ j k l|i ] h
3 [|m n     ]
```

move split
```
    0 1 2 3
0 [ c d|a b ]
1 [ e|f x g ]
   |<<
2 [ j k l|i ] h
3 [|m n     ]
```

swap into next row
```
    0 1 2 3
0 [ c d|a b ]
1 [|e f x g ]
2 [ j k l|i ] h
        ^     ^
3 [|m n     ]
```

move split
```
    0 1 2 3
0 [ c d|a b ]
1 [|e f x g ]
2 [ j k h|i ]
       |<<
3 [|m n     ] l
```

insert at front of last DEQ
```
    0 1 2 3
0 [ c d|a b ]
1 [|e f x g ]
2 [ j k|h i ]
3 [|m n     ] l
   I
```

done
```
    0 1 2 3
0 [ c d|a b ]
1 [|e f x g ]
2 [ j k|h i ]
3 [|l m n   ]
```
