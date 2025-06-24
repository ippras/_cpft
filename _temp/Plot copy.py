import pandas as pd
import plotly.express as px

# Load the dataset
df = pd.read_csv('_temp/experiment.csv')

# Drop rows with missing values in the key columns for plotting
df_cleaned = df.dropna(subset=['ECL', 'OnsetTemperature', 'TemperatureStep'])

# Filter out rows where ECL is a whole number
df_filtered = df_cleaned[df_cleaned['ECL'] % 1 != 0].copy()

# Convert TemperatureStep to string to ensure it's treated as a discrete category for symbols
df_filtered['TemperatureStep'] = df_filtered['TemperatureStep'].astype(str)

# Create the interactive 3D scatter plot
fig = px.scatter_3d(
    df_filtered,
    x='ECL',
    y='OnsetTemperature',
    z='TemperatureStep',
    color='OnsetTemperature',
    symbol='TemperatureStep',
    title='Интерактивный 3D-график с данными при наведении',
    labels={
        'ECL': 'ECL',
        'OnsetTemperature': 'Onset Temperature',
        'TemperatureStep': 'Temperature Step'
    },
    hover_data=df.columns  # This line includes all columns in the hover tooltip
)

# Update layout for better appearance
fig.update_layout(
    margin=dict(l=0, r=0, b=0, t=40),
    legend_title_text='Temperature Step'
)

# Show the figure
fig.show()
