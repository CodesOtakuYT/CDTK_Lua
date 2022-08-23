local x = CDTK.VecF64()
x:range(0, 10, 1)
print(x)
print(x:transform("x*2"):eval())