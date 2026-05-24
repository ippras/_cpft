import pandas as pd
import plotly.express as px

# Load the dataset
df = pd.read_csv('_temp/experiment.csv')

# Drop rows with missing values
df_cleaned = df.dropna(subset=['ECL', 'OnsetTemperature', 'TemperatureStep'])

# Filter out rows where ECL is a whole number
df_filtered = df_cleaned[df_cleaned['ECL'] % 1 != 0].copy()

# Convert TemperatureStep to string for discrete symbols
df_filtered['TemperatureStep'] = df_filtered['TemperatureStep'].astype(str)

# Create the interactive 3D scatter plot
fig = px.scatter_3d(
    df_filtered,
    x='ECL',
    y='OnsetTemperature',
    z='TemperatureStep',
    color='OnsetTemperature',
    symbol='TemperatureStep',
    title='Интерактивный 3D-график',
    labels={
        'ECL': 'ECL',
        'OnsetTemperature': 'Onset Temperature',
        'TemperatureStep': 'Temperature Step'
    },
    hover_data=df.columns,
    color_continuous_scale=px.colors.sequential.Viridis # Explicitly set a nice color scale
)

# Update layout to fix legend overlap
fig.update_layout(
    margin=dict(l=0, r=0, b=0, t=40),
    legend=dict(
        yanchor="top",
        y=0.9,
        xanchor="left",
        x=0.01,
        title="Temperature Step"
    )
)

# Show the figure (this will be interactive in your environment)
fig.show()
