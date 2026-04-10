import pandas as pd
import matplotlib.pyplot as plt
import io

data = """A,B,C,D
8,23.365,0.249,85.229
10,39.027,0.416,84.852
11,47.48,0.506,84.873
12,55.897,0.596,85.01
13,64.088,0.683,85.18
14,72,0.767,85.382
15,79.579,0.848,85.546
16,86.887,0.926,85.774
17,93.82,1,85.907
18,100.534,1.072,86.096
19,106.864,1.14,86.236
20,113.104,1.206,86.418
21,118.977,1.268,86.476
22,124.754,1.33,86.684
23,130.19,1.388,86.703
"""

df = pd.read_csv(io.StringIO(data))

plt.figure(figsize=(12, 8))

plt.subplot(3, 1, 1)
plt.plot(df['A'], df['B'], marker='o')
plt.title('График B от A')
plt.xlabel('A')
plt.ylabel('B')
plt.grid(True)

plt.subplot(3, 1, 2)
plt.plot(df['A'], df['C'], marker='o', color='orange')
plt.title('График C от A')
plt.xlabel('A')
plt.ylabel('C')
plt.grid(True)

plt.subplot(3, 1, 3)
plt.plot(df['A'], df['D'], marker='o', color='green')
plt.title('График D от A')
plt.xlabel('A')
plt.ylabel('D')
plt.grid(True)

plt.tight_layout()
plt.show()