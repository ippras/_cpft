import pandas as pd
from mpl_toolkits.mplot3d import Axes3D
import matplotlib.pyplot as plt

# Load the dataset
df = pd.read_csv('_temp/experiment.csv')

# Drop rows with missing values in the relevant columns for plotting
df_cleaned = df.dropna(subset=['ECL', 'OnsetTemperature', 'TemperatureStep'])

# Create a 3D plot
fig = plt.figure(figsize=(10, 8))
ax = fig.add_subplot(111, projection='3d')

# Scatter plot
ax.scatter(df_cleaned['ECL'], df_cleaned['OnsetTemperature'], df_cleaned['TemperatureStep'], c='r', marker='o')

# Set labels
ax.set_xlabel('ECL')
ax.set_ylabel('OnsetTemperature')
ax.set_zlabel('TemperatureStep')

# Show plot
plt.show()
