import pandas as pd
import plotly.express as px

# Load the new dataset
df = pd.read_csv('_temp/source.csv')

# Drop rows where EquivalentChainLength is missing, as they cannot be plotted.
df_cleaned = df.dropna(subset=['EquivalentChainLength'])

# Filter out rows where ECL is a whole number
df_filtered = df_cleaned[df_cleaned['EquivalentChainLength'] % 1 != 0].copy()

# Create a copy for modifications to avoid SettingWithCopyWarning
df_plot = df_filtered.copy()

# --- Key Change: Create NEW columns for categorical data ---
# This avoids the name ambiguity error. The original columns remain numeric.
df_plot['OnsetTemperature_cat'] = df_plot['OnsetTemperature'].astype(str)
df_plot['TemperatureStep_cat'] = df_plot['TemperatureStep'].astype(str)

# Create the interactive 3D scatter plot
fig = px.scatter_3d(
    df_plot,
    x='EquivalentChainLength',
    y='OnsetTemperature',  # Use the original numeric column for the axis value
    z='TemperatureStep',   # Use the original numeric column for the axis value
    color='OnsetTemperature_cat',  # Use the categorical column for the legend
    symbol='TemperatureStep_cat', # Use the categorical column for the legend
    title='Интерактивный 3D-график с кликабельными легендами',
    labels={
        'EquivalentChainLength': 'Equivalent Chain Length',
        'OnsetTemperature': 'Onset Temperature',
        'TemperatureStep': 'Temperature Step',
        'OnsetTemperature_cat': 'Onset Temperature', # Label for the color legend
        'TemperatureStep_cat': 'Temperature Step'   # Label for the symbol legend
    },
    # Customize what data appears on hover
    hover_data={
        'FattyAcid': True,
        'EquivalentChainLength': ':.2f',
        # The main axes data is already included, so we don't need to re-add it.
        # We hide the categorical columns from the hover tooltip.
        'OnsetTemperature_cat': False,
        'TemperatureStep_cat': False
    }
)

# Update layout for better legend placement and appearance
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
