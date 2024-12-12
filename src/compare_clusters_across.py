#!/usr/bin/python


handle = open("campy_poppunk/campy_poppunk_clusters.csv", 'r')
infile = open("gyrB_res_cluster.csv", 'r')


d = {}

for line in handle:
    elems = line.rstrip().split(',')
    try:
        d[elems[0]] = elems[1]
    except:
        d[elems[0]] = elems[1]


for lin in infile:
    items = lin.rstrip().split(',')
    print("{},{},{}".format(items[0],items[1],d[items[0]]))
