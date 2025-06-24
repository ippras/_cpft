import pandas as pd
import plotly.express as px

# Load the dataset
df = pd.read_csv('_temp/experiment.csv')

# Drop rows with missing values
df_cleaned = df.dropna(subset=['ECL', 'OnsetTemperature', 'TemperatureStep'])

# Filter out rows where ECL is a whole number
df_filtered = df_cleaned[df_cleaned['ECL'] % 1 != 0].copy()

# --- Key Change: Bin OnsetTemperature into discrete groups ---
# This creates a new categorical column that can be used for a clickable legend.
df_filtered['OnsetTemperature_Group'] = pd.cut(
    df_filtered['OnsetTemperature'],
    bins=10, # We can adjust the number of groups here
    precision=0 # Controls the decimal precision of the bin labels
)

# Convert TemperatureStep to string for discrete symbols
df_filtered['TemperatureStep'] = df_filtered['TemperatureStep'].astype(str)

# Create the interactive 3D scatter plot
fig = px.scatter_3d(
    df_filtered,
    x='ECL',
    y='OnsetTemperature',
    z='TemperatureStep',
    color='OnsetTemperature_Group',  # Use the new binned column for color
    symbol='TemperatureStep',
    title='Интерактивный 3D-график с кликабельными легендами',
    labels={
        'ECL': 'ECL',
        'OnsetTemperature': 'Onset Temperature',
        'TemperatureStep': 'Temperature Step',
        'OnsetTemperature_Group': 'Onset Temperature Group'
    },
    hover_data={
        'OnsetTemperature': ':.2f', # Show precise temperature on hover
        'TimeMean': ':.2f',
        'ECL': ':.2f',
        'FCL': ':.2f',
        'ECN': True
    }
)

# Update layout for better legend placement
fig.update_layout(
    margin=dict(l=0, r=0, b=0, t=40),
    legend=dict(
        title="Фильтры",
        yanchor="top",
        y=0.99,
        xanchor="left",
        x=0.01
    )
)

# Show the figure (this will be interactive in your environment)
fig.show()
