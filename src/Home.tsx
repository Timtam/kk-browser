import { invoke } from "@tauri-apps/api/core"
import natsort from "natsort"
import { useEffect, useMemo, useState } from "react"
import Button from "react-bootstrap/Button"
import Form from "react-bootstrap/Form"
import Modal from "react-bootstrap/Modal"
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

interface Category {
    id: number
    name: string
    subcategory: string
    subsubcategory: string
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
    const [temporarilySelectedVendors, setTemporarilySelectedVendors] =
        useState<string[]>([])
    const [products, setProducts] = useState<Map<number, Product>>(new Map())
    const [selectedProducts, setSelectedProducts] = useState<number[]>([])
    const [temporarilySelectedProducts, setTemporarilySelectedProducts] =
        useState<number[]>([])
    const sorter = useMemo(natsort, [])
    const [preset, setSelectedPreset] = useState<PresetOption | undefined>(
        undefined,
    )
    const [categories, setCategories] = useState<Map<number, Category>>(
        new Map(),
    )
    const [selectedCategories, setSelectedCategories] = useState<number[]>([])
    const [temporarilySelectedCategories, setTemporarilySelectedCategories] =
        useState<number[]>([])
    const [showProducts, setShowProducts] = useState(false)
    const [showVendors, setShowVendors] = useState(false)
    const [showCategories, setShowCategories] = useState(false)

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
                                categories: selectedCategories,
                            })) as Product[]
                        ).map((p) => [p.id, p]),
                    ),
                )
                setCategories(
                    new Map(
                        (
                            (await invoke("get_categories", {
                                vendors: selectedVendors,
                                products: selectedProducts,
                            })) as Category[]
                        ).map((c) => [c.id, c]),
                    ),
                )
            }
        })()
    }, [
        loading,
        selectedCategories,
        selectedProducts,
        selectedVendors,
        setCategories,
        setProducts,
    ])

    return loading ? (
        <p>Loading Komplete Kontrol data, please wait...</p>
    ) : (
        <>
            <Button aria-expanded={false} onClick={() => setShowVendors(true)}>
                Vendors:{" "}
                {selectedVendors.length === 0
                    ? "All"
                    : joinString(selectedVendors.sort(sorter), ", ", " and ")}
            </Button>
            <Modal
                show={showVendors}
                onHide={() => {
                    setShowVendors(false)
                    setSelectedVendors(temporarilySelectedVendors)
                }}
            >
                <Modal.Header closeButton closeLabel="Save">
                    <Modal.Title>Vendors</Modal.Title>
                </Modal.Header>
                <Modal.Body>
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
                                        checked={temporarilySelectedVendors.includes(
                                            v,
                                        )}
                                        onChange={() =>
                                            temporarilySelectedVendors.includes(
                                                v,
                                            )
                                                ? setTemporarilySelectedVendors(
                                                      temporarilySelectedVendors.filter(
                                                          (v2) => v !== v2,
                                                      ),
                                                  )
                                                : setTemporarilySelectedVendors(
                                                      [
                                                          ...temporarilySelectedVendors,
                                                          v,
                                                      ],
                                                  )
                                        }
                                    />
                                </div>
                            ))}
                    </div>
                </Modal.Body>
            </Modal>
            <Button aria-expanded={false} onClick={() => setShowProducts(true)}>
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
            </Button>
            <Modal
                show={showProducts}
                onHide={() => {
                    setShowProducts(false)
                    setSelectedProducts(temporarilySelectedProducts)
                }}
            >
                <Modal.Header closeButton closeLabel="Save">
                    <Modal.Title>Products</Modal.Title>
                </Modal.Header>
                <Modal.Body>
                    <div role="list" aria-label="Products">
                        {[...products!.values()].map((p, i) => (
                            <div role="listitem">
                                <Form.Check
                                    type="checkbox"
                                    id={`${slugify(p.name)}-${i}`}
                                    label={p.name}
                                    checked={temporarilySelectedProducts.includes(
                                        p.id,
                                    )}
                                    onChange={() =>
                                        temporarilySelectedProducts.includes(
                                            p.id,
                                        )
                                            ? setTemporarilySelectedProducts(
                                                  temporarilySelectedProducts.filter(
                                                      (p2) => p.id !== p2,
                                                  ),
                                              )
                                            : setTemporarilySelectedProducts([
                                                  ...temporarilySelectedProducts,
                                                  p.id,
                                              ])
                                    }
                                />
                            </div>
                        ))}
                    </div>
                </Modal.Body>
            </Modal>
            <Button
                aria-expanded={false}
                onClick={() => setShowCategories(true)}
            >
                Categories:{" "}
                {selectedCategories.length === 0
                    ? "All"
                    : joinString(
                          selectedCategories
                              .map((c) =>
                                  joinString(
                                      [
                                          categories.get(c)!.name,
                                          categories.get(c)!.subcategory,
                                          categories.get(c)!.subsubcategory,
                                      ].filter((c) => c !== ""),
                                      " / ",
                                  ),
                              )
                              .sort(sorter),
                          ", ",
                          " and ",
                      )}
            </Button>
            <Modal
                show={showCategories}
                onHide={() => {
                    setShowCategories(false)
                    setSelectedCategories(temporarilySelectedCategories)
                }}
            >
                <Modal.Header closeButton closeLabel="Save">
                    <Modal.Title>Categories</Modal.Title>
                </Modal.Header>
                <Modal.Body>
                    <div role="list" aria-label="Categories">
                        {[...categories!.values()].map((c, i) => (
                            <div role="listitem">
                                <Form.Check
                                    type="checkbox"
                                    id={`${slugify(c.name)}-${i}`}
                                    label={joinString(
                                        [
                                            c.name,
                                            c.subcategory,
                                            c.subsubcategory,
                                        ].filter((c) => c !== ""),
                                        " / ",
                                    )}
                                    checked={temporarilySelectedCategories.includes(
                                        c.id,
                                    )}
                                    onChange={() =>
                                        temporarilySelectedCategories.includes(
                                            c.id,
                                        )
                                            ? setTemporarilySelectedCategories(
                                                  temporarilySelectedCategories.filter(
                                                      (c2) => c.id !== c2,
                                                  ),
                                              )
                                            : setTemporarilySelectedCategories([
                                                  ...temporarilySelectedCategories,
                                                  c.id,
                                              ])
                                    }
                                />
                            </div>
                        ))}
                    </div>
                </Modal.Body>
            </Modal>
            <Select
                closeMenuOnSelect={false}
                cacheUniqs={[
                    selectedCategories,
                    selectedProducts,
                    selectedVendors,
                ]}
                value={preset}
                isMulti={false}
                isSearchable={true}
                loadOptions={async (_: string, loadedOptions) => {
                    let res = (await invoke("get_presets", {
                        vendors: selectedVendors,
                        products: selectedProducts,
                        categories: selectedCategories,
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
