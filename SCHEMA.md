# Schema

## 0.2

```
Schema {
    fields: {
        "Index": UInt32,
        "Mode": Struct({'OnsetTemperature': Float64, 'TemperatureStep': Float64}),
        "FattyAcid": Struct({'Carbon': UInt8, 'Indices': List(Struct({'Index': UInt8, 'Triple': Boolean, 'Parity': Boolean}))}),
        "RetentionTime": List(Float64),
        "DeadTime": Float64,
    },
    metadata: (),
}
```

## 0.3

```
Schema {
    fields: {
        "Index": UInt32,
        "Mode": Struct({'OnsetTemperature': Float64, 'TemperatureStep': Float64}),
        "FattyAcid": Struct({'Carbon': UInt8, 'Indices': List(Struct({'Index': UInt8, 'Triple': Boolean, 'Parity': Boolean}))}),
        "RetentionTime": Array(Float64, 3),
        "DeadTime": Float64,
    },
    metadata: (),
}
```
