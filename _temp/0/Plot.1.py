import pandas as pd
import matplotlib.pyplot as plt
from mpl_toolkits.mplot3d import Axes3D
import numpy as np

# Load the dataset
df = pd.read_csv('_temp/experiment.csv')

# Drop rows with missing values
df_cleaned = df.dropna(subset=['ECL', 'OnsetTemperature', 'TemperatureStep'])

# Get unique values for TemperatureStep
unique_temp_steps = sorted(df_cleaned['TemperatureStep'].unique())
colors = plt.get_cmap('tab10')(np.linspace(0, 1, len(unique_temp_steps)))
markers = ['o', 's', '^', 'D', 'v', '<', '>', 'p', '*', 'h']

# Create a 3D plot
fig = plt.figure(figsize=(12, 10))
ax = fig.add_subplot(111, projection='3d')

# Plot each group with a different color and marker
for i, step in enumerate(unique_temp_steps):
    subset = df_cleaned[df_cleaned['TemperatureStep'] == step]
    ax.scatter(subset['ECL'], subset['OnsetTemperature'], subset['TemperatureStep'], 
               color=colors[i], 
               marker=markers[i % len(markers)], 
               label=f'Step = {step}')

# Set labels and title
ax.set_xlabel('ECL')
ax.set_ylabel('OnsetTemperature')
ax.set_zlabel('TemperatureStep')
ax.set_title('3D Plot of ECL, OnsetTemperature, and TemperatureStep')

# Add a legend
ax.legend(title='Temperature Step')

# Show plot
plt.show()
