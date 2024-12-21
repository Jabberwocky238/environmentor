
the basic component of ehat i gonna say is like:

```js
<input
    onChange={(e) => setBuffer(e.currentTarget.value)}
    placeholder="Enter value"
    value={buffer}
/>
```

In react, after typing in something, value will change.

now see this:

```typescript
<div className="list">
    {valueList.map((v, i) => (
        <>
            <div className="value-item"
                style={{ display: i === curEditValIndex ? "none" : "block" }}
                onClick={() => {
                    setEditValIndex(i);
                    setBuffer(v);
                }}>{v}</div>

            <div className="value-item-editing"
                style={{ display: i === curEditValIndex ? "flex" : "none" }}>
                <button onClick={() => btnOrder('up')}>↑</button>
                <button onClick={() => btnOrder('down')}>↓</button>

                <input
                    onChange={(e) => setBuffer(e.currentTarget.value)}
                    placeholder="Enter value"
                    value={buffer}
                />

                <button onClick={btnModifyConform}>Conform</button>
                {/* <button onClick={btnFromFS}>FromFS</button> */}
                <button onClick={btnDelete}>Delete</button>
            </div>
        </>
    ))}

    <div className="value-item-editing"
        style={{ display: isAddValueOpen ? "flex" : "none" }}>
        <input
            onChange={(e) => setBuffer(e.currentTarget.value)}
            placeholder="Enter value"
            value={buffer}
        />
        <button onClick={btnAddConform}>+</button>
    </div>
</div>
```
"@tauri-apps/api": "^2",
"@tauri-apps/plugin-dialog": "~2",
"react": "^18.2.0",

The function of this code

there is 2 parts of this code
1, render a list for modification of each item
2, render an additional input for adding items

in the second situation, there is no problem

but in the first one, I cannot type in any text
especially speaking
when i am using English input method, cursor will loss focus of the input element after one letter typed in.
when i am using 'Simplified Chinese' input method, I cannot type in any letter, even in English mode.

```
// import { open } from '@tauri-apps/plugin-dialog';
```
after i stop using this plugin, everything works fine
but if i uncomment this, even i dont use it, it will cause the bug.



