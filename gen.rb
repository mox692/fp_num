#
# 2^(-n) を表示するscript
#

for i in 1..23
    cur = 1.0 / (2**i)
    inc = 0
    while cur - cur.to_i != 0 do
        cur *= 10
        inc+=1
    end
    printf("%du32 => Internal(%d, %d),\n", i, cur.to_i, inc)
end
