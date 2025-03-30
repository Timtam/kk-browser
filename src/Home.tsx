import { invoke } from "@tauri-apps/api/core"
import natsort from "natsort"
import { useEffect, useMemo, useState } from "react"
import Accordion from "react-bootstrap/Accordion"
import Form from "react-bootstrap/Form"
import { AsyncPaginate as Select } from "react-select-async-paginate"
import slugify from "slugify"
import { joinString } from "./utils"

const PAGE_SIZE = 500

interface Preset {
    name: string
    comment: string
    vendor: string
    product_name: String
    id: number
}

interface Product {
    name: string
    vendor: string
    id: number
}

interface PaginatedResult<T> {
    results: T[]
    total: number
    start: number
    end: number
}

interface PresetOption extends Preset {
    label: string
}

function Home() {
    const [loading, setLoading] = useState(true)
    const [vendors, setVendors] = useState<string[]>([])
    const [selectedVendors, setSelectedVendors] = useState<string[]>([])
    const [products, setProducts] = useState<Map<number, Product>>(new Map())
    const [selectedProducts, setSelectedProducts] = useState<number[]>([])
    const sorter = useMemo(natsort, [])
    const [preset, setSelectedPreset] = useState<PresetOption | undefined>(
        undefined,
    )

    useEffect(() => {
        ;(async () => {
            if (loading) {
                let server_loading = true

                while (server_loading) {
                    server_loading = await invoke("is_loading")
                    if (server_loading)
                        await new Promise((r) => setTimeout(r, 100))
                }
                setVendors(await invoke("get_vendors"))
                setLoading(false)
            }
        })()
    }, [loading, setLoading, setVendors])

    useEffect(() => {
        ;(async () => {
            if (!loading) {
                setProducts(
                    new Map(
                        (
                            (await invoke("get_products", {
                                vendors: selectedVendors,
                            })) as Product[]
                        ).map((p) => [p.id, p]),
                    ),
                )
            }
        })()
    }, [loading, selectedVendors, setProducts])

    return loading ? (
        <p>Loading Komplete Kontrol data, please wait...</p>
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
                                            key={`${slugify(v)}-${i}`}
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
            <Select
                closeMenuOnSelect={false}
                cacheUniqs={[selectedProducts, selectedVendors]}
                value={preset}
                isMulti={false}
                isSearchable={true}
                loadOptions={async (_: string, loadedOptions) => {
                    let res = (await invoke("get_presets", {
                        vendors: selectedVendors,
                        products: selectedProducts,
                        offset: loadedOptions.length,
                        limit: PAGE_SIZE,
                    })) as PaginatedResult<Preset>

                    return {
                        options: res.results.map((p) => ({
                            ...p,
                            label: `${p.name}, ${p.comment}, ${p.product_name}`,
                        })),
                        hasMore: res.total > res.end,
                    }
                }}
                aria-label="Presets"
                onChange={(o) => {
                    ;(async () => {
                        setSelectedPreset(o!)
                        await invoke("play_preset", {
                            preset: o!.id,
                        })
                    })()
                }}
            />
        </>
    )
}

export default Home
