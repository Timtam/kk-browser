import { invoke } from "@tauri-apps/api/core"
import natsort from "natsort"
import { useEffect, useMemo, useState } from "react"
import Accordion from "react-bootstrap/Accordion"
import Form from "react-bootstrap/Form"
import slugify from "slugify"
import { joinString } from "./utils"

const PAGE_SIZE = 50

interface Preset {
    name: string
    comment: string
    vendor: string
    product: String
    id: number
}

interface Product {
    name: string
    vendor: string
    id: number
}

function Home() {
    const [loading, setLoading] = useState(true)
    const [vendors, setVendors] = useState<string[]>([])
    const [selectedVendors, setSelectedVendors] = useState<string[]>([])
    const [products, setProducts] = useState<Map<number, Product>>(new Map())
    const [selectedProducts, setSelectedProducts] = useState<number[]>([])
    const [presets, setPresets] = useState<Preset[]>([])
    const [selectedPreset, setSelectedPreset] = useState(0)
    const [offset, setOffset] = useState(0)
    const sorter = useMemo(natsort, [])

    useEffect(() => {
        ;(async () => {
            setVendors(await invoke("get_vendors"))
            setProducts(
                new Map(
                    (
                        (await invoke("get_products", {
                            vendors: [],
                        })) as Product[]
                    ).map((p) => [p.id, p]),
                ),
            )
            const p = (await invoke("get_presets", {
                vendors: [],
                products: [],
                offset: 0,
                limit: PAGE_SIZE,
            })) as Preset[]
            setPresets(p)
            setSelectedPreset(p[0].id)
            setLoading(false)
        })()
    }, [setLoading, setPresets, setProducts, setVendors])

    useEffect(() => {
        ;(async () => {
            setProducts(
                new Map(
                    (
                        (await invoke("get_products", {
                            vendors: selectedVendors,
                        })) as Product[]
                    ).map((p) => [p.id, p]),
                ),
            )
            const p = (await invoke("get_presets", {
                vendors: selectedVendors,
                products: selectedProducts,
                offset: 0,
                limit: PAGE_SIZE,
            })) as Preset[]
            setOffset(0)
            setSelectedPreset(p[0].id)
            setPresets(p)
        })()
    }, [
        selectedProducts,
        selectedVendors,
        setOffset,
        setPresets,
        setProducts,
        setSelectedPreset,
    ])

    return loading ? (
        <p>Loading...</p>
    ) : (
        <>
            <Accordion>
                <Accordion.Item eventKey="vendors">
                    <Accordion.Header as="p">
                        Vendors:{" "}
                        {selectedVendors.length === 0
                            ? "All"
                            : joinString(
                                  selectedVendors.sort(sorter),
                                  ", ",
                                  " and ",
                              )}
                    </Accordion.Header>
                    <Accordion.Body>
                        <div role="list" aria-label="Vendors">
                            {vendors!
                                .filter(
                                    (v) =>
                                        selectedProducts.length === 0 ||
                                        selectedProducts
                                            .map((p) => products.get(p)!)
                                            .find((p) => p.vendor === v) !==
                                            undefined,
                                )
                                .sort(sorter)
                                .map((v, i) => (
                                    <div role="listitem">
                                        <Form.Check
                                            type="checkbox"
                                            id={`${slugify(v)}-${i}`}
                                            label={v}
                                            checked={selectedVendors.includes(
                                                v,
                                            )}
                                            onChange={() =>
                                                selectedVendors.includes(v)
                                                    ? setSelectedVendors(
                                                          selectedVendors.filter(
                                                              (v2) => v !== v2,
                                                          ),
                                                      )
                                                    : setSelectedVendors([
                                                          ...selectedVendors,
                                                          v,
                                                      ])
                                            }
                                        />
                                    </div>
                                ))}
                        </div>
                    </Accordion.Body>
                </Accordion.Item>
            </Accordion>
            <Accordion>
                <Accordion.Item eventKey="products">
                    <Accordion.Header as="p">
                        Products:{" "}
                        {selectedProducts.length === 0
                            ? "All"
                            : joinString(
                                  selectedProducts
                                      .map((p) => products.get(p)!.name)
                                      .sort(sorter),
                                  ", ",
                                  " and ",
                              )}
                    </Accordion.Header>
                    <Accordion.Body>
                        <div role="list" aria-label="Products">
                            {[...products!.values()].map((p, i) => (
                                <div role="listitem">
                                    <Form.Check
                                        type="checkbox"
                                        id={`${slugify(p.name)}-${i}`}
                                        label={p.name}
                                        checked={selectedProducts.includes(
                                            p.id,
                                        )}
                                        onChange={() =>
                                            selectedProducts.includes(p.id)
                                                ? setSelectedProducts(
                                                      selectedProducts.filter(
                                                          (p2) => p.id !== p2,
                                                      ),
                                                  )
                                                : setSelectedProducts([
                                                      ...selectedProducts,
                                                      p.id,
                                                  ])
                                        }
                                    />
                                </div>
                            ))}
                        </div>
                    </Accordion.Body>
                </Accordion.Item>
            </Accordion>
            <select
                aria-label="Presets"
                onChange={async (e) => {
                    setSelectedPreset(parseInt(e.currentTarget.value, 10))
                    await invoke("play_preset", {
                        preset: parseInt(e.currentTarget.value, 10),
                    })
                    if (
                        presets.findIndex(
                            (p) => p.id === parseInt(e.currentTarget.value, 10),
                        ) >
                        offset - 10
                    ) {
                        const p = (await invoke("get_presets", {
                            vendors: selectedVendors,
                            products: selectedProducts,
                            offset: offset,
                            limit: PAGE_SIZE,
                        })) as Preset[]
                        setOffset(offset + p.length)
                        setPresets((old_presets) => old_presets.concat(p))
                    }
                }}
            >
                {presets.map((p) => {
                    return (
                        <option
                            key={p.id}
                            selected={p.id === selectedPreset}
                            id={p.id.toString()}
                            value={p.id}
                        >
                            {`${p.name}, ${p.comment}, ${p.product}, ${p.vendor}`}
                        </option>
                    )
                })}
            </select>
        </>
    )
}

export default Home
