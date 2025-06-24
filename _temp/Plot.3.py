import pandas as pd
import matplotlib.pyplot as plt
from mpl_toolkits.mplot3d import Axes3D
import numpy as np
from matplotlib.lines import Line2D

# Load the dataset
df = pd.read_csv('_temp/experiment.csv')

# Drop rows with missing values
df_cleaned = df.dropna(subset=['ECL', 'OnsetTemperature', 'TemperatureStep'])

# Filter out rows where ECL is a whole number
df_filtered = df_cleaned[df_cleaned['ECL'] % 1 != 0]

# --- Plotting ---
fig = plt.figure(figsize=(14, 12))
ax = fig.add_subplot(111, projection='3d')

# Define markers for each TemperatureStep
unique_temp_steps = sorted(df_filtered['TemperatureStep'].unique())
markers = ['o', 's', '^', 'D', 'v', '<', '>', 'p', '*', 'h']
marker_map = {step: markers[i % len(markers)] for i, step in enumerate(unique_temp_steps)}

# Define the colormap for OnsetTemperature
cmap = plt.get_cmap('viridis')

# Plot the data
sc_plot = None
for step in unique_temp_steps:
    subset = df_filtered[df_filtered['TemperatureStep'] == step]
    if not subset.empty:
        sc_plot = ax.scatter(subset['ECL'], subset['OnsetTemperature'], subset['TemperatureStep'],
                             c=subset['OnsetTemperature'],
                             cmap=cmap,
                             marker=marker_map[step],
                             s=50,
                             alpha=0.8)

# Set labels and title
ax.set_xlabel('ECL')
ax.set_ylabel('OnsetTemperature')
ax.set_zlabel('TemperatureStep')
ax.set_title('3D-график (ECL не целое): цвет по OnsetTemperature, маркер по TemperatureStep')

# Create a colorbar for OnsetTemperature
if sc_plot:
    cbar = fig.colorbar(sc_plot, ax=ax, shrink=0.6, aspect=20, pad=0.1)
    cbar.set_label('OnsetTemperature')

# Create a custom legend for the markers
legend_elements = [Line2D([0], [0], marker=marker_map[step], color='grey', label=f'Step = {step}',
                          markerfacecolor='grey', markersize=8, linestyle='None')
                   for step in unique_temp_steps]
ax.legend(handles=legend_elements, title='Temperature Step', bbox_to_anchor=(1.1, 0.7))

# Adjust layout to prevent labels from overlapping
plt.tight_layout()

# Show plot
plt.show()
