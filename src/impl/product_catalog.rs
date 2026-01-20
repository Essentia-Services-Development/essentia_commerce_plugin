//! # Product Catalog (GAP-220-D-001)
//!
//! Complete product catalog management system for the e-commerce platform.

use std::{
    borrow::Cow,
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::errors::CommerceError;

// ============================================================================
// CORE TYPES
// ============================================================================

/// Unique product identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProductId(pub Cow<'static, str>);

impl ProductId {
    /// Creates a new product ID.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(Cow::Owned(id.into()))
    }

    /// Creates a product ID from a static string slice (zero-copy).
    #[must_use]
    pub fn from_static(id: &'static str) -> Self {
        Self(Cow::Borrowed(id))
    }

    /// Returns the ID as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ProductId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Category identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CategoryId(pub Cow<'static, str>);

impl std::fmt::Display for CategoryId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl CategoryId {
    /// Creates a new category ID.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(Cow::Owned(id.into()))
    }

    /// Creates a category ID from a static string slice (zero-copy).
    #[must_use]
    pub fn from_static(id: &'static str) -> Self {
        Self(Cow::Borrowed(id))
    }
}

/// Unique SKU (Stock Keeping Unit).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Sku(pub Cow<'static, str>);

impl std::fmt::Display for Sku {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Sku {
    /// Creates a new SKU.
    #[must_use]
    pub fn new(sku: impl Into<String>) -> Self {
        Self(Cow::Owned(sku.into()))
    }

    /// Creates a SKU from a static string slice (zero-copy).
    #[must_use]
    pub fn from_static(sku: &'static str) -> Self {
        Self(Cow::Borrowed(sku))
    }
}

/// Product status in the catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProductStatus {
    /// Product is active and available.
    #[default]
    Active,
    /// Product is inactive but not deleted.
    Inactive,
    /// Product is a draft (not published).
    Draft,
    /// Product is archived.
    Archived,
    /// Product is discontinued.
    Discontinued,
    /// Product is out of stock.
    OutOfStock,
    /// Product is pending approval.
    PendingApproval,
    /// Product is deleted (soft delete).
    Deleted,
}

impl ProductStatus {
    /// Whether the product is visible to customers.
    #[must_use]
    pub fn is_visible(&self) -> bool {
        matches!(self, Self::Active | Self::OutOfStock)
    }

    /// Whether the product can be purchased.
    #[must_use]
    pub fn is_purchasable(&self) -> bool {
        matches!(self, Self::Active)
    }
}

/// Product type classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProductType {
    /// Physical product requiring shipping.
    #[default]
    Physical,
    /// Digital product (download).
    Digital,
    /// Service offering.
    Service,
    /// Subscription-based product.
    Subscription,
    /// Bundle of multiple products.
    Bundle,
    /// Gift card or voucher.
    GiftCard,
    /// Configurable product with variants.
    Configurable,
    /// Virtual goods (in-game items, etc.).
    Virtual,
}

/// Currency code (ISO 4217).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Currency(pub String);

impl Currency {
    /// Creates a new currency code.
    #[must_use]
    pub fn new(code: impl Into<String>) -> Self {
        Self(code.into())
    }

    /// Essentia native token.
    #[must_use]
    pub fn ess() -> Self {
        Self("ESS".to_string())
    }

    /// US Dollar.
    #[must_use]
    pub fn usd() -> Self {
        Self("USD".to_string())
    }
}

/// Price with currency.
#[derive(Debug, Clone, PartialEq)]
pub struct Price {
    /// Amount in smallest currency unit (cents, satoshi, etc.).
    pub amount:   u64,
    /// Currency code.
    pub currency: Currency,
    /// Number of decimal places.
    pub decimals: u8,
}

impl Price {
    /// Creates a new price.
    #[must_use]
    pub fn new(amount: u64, currency: Currency, decimals: u8) -> Self {
        Self { amount, currency, decimals }
    }

    /// Creates a price in ESS tokens.
    #[must_use]
    pub fn ess(amount: u64) -> Self {
        Self::new(amount, Currency::ess(), 18)
    }

    /// Returns the display amount (with decimals applied).
    #[must_use]
    pub fn display_amount(&self) -> f64 {
        let divisor = 10_u64.pow(u32::from(self.decimals));
        self.amount as f64 / divisor as f64
    }

    /// Adds another price (must be same currency).
    ///
    /// # Errors
    /// Returns error if currencies don't match.
    pub fn add(&self, other: &Price) -> Result<Price, CommerceError> {
        if self.currency != other.currency {
            return Err(CommerceError::CurrencyMismatch {
                expected: self.currency.0.to_string(),
                got:      other.currency.0.to_string(),
            });
        }
        Ok(Price::new(
            self.amount.saturating_add(other.amount),
            self.currency.clone(),
            self.decimals,
        ))
    }
}

impl Default for Price {
    fn default() -> Self {
        Self::ess(0)
    }
}

// ============================================================================
// PRODUCT METADATA
// ============================================================================

/// Product dimensions for shipping.
#[derive(Debug, Clone, Default)]
pub struct ProductDimensions {
    /// Length in centimeters.
    pub length_cm:    f32,
    /// Width in centimeters.
    pub width_cm:     f32,
    /// Height in centimeters.
    pub height_cm:    f32,
    /// Weight in grams.
    pub weight_grams: u32,
}

impl ProductDimensions {
    /// Creates new dimensions.
    #[must_use]
    pub fn new(length_cm: f32, width_cm: f32, height_cm: f32, weight_grams: u32) -> Self {
        Self { length_cm, width_cm, height_cm, weight_grams }
    }

    /// Calculates volumetric weight for shipping.
    #[must_use]
    pub fn volumetric_weight(&self, divisor: f32) -> f32 {
        (self.length_cm * self.width_cm * self.height_cm) / divisor
    }
}

/// Product image information.
#[derive(Debug, Clone)]
pub struct ProductImage {
    /// Image URL or content hash.
    pub url:        String,
    /// Alternative text for accessibility.
    pub alt_text:   String,
    /// Sort order in gallery.
    pub sort_order: u32,
    /// Whether this is the main product image.
    pub is_primary: bool,
    /// Image width in pixels.
    pub width:      Option<u32>,
    /// Image height in pixels.
    pub height:     Option<u32>,
}

impl ProductImage {
    /// Creates a new product image.
    #[must_use]
    pub fn new(url: impl Into<String>, alt_text: impl Into<String>) -> Self {
        Self {
            url:        url.into(),
            alt_text:   alt_text.into(),
            sort_order: 0,
            is_primary: false,
            width:      None,
            height:     None,
        }
    }

    /// Marks this image as primary.
    #[must_use]
    pub fn as_primary(mut self) -> Self {
        self.is_primary = true;
        self
    }
}

/// Product attribute (configurable properties).
#[derive(Debug, Clone)]
pub struct ProductAttribute {
    /// Attribute name (e.g., "Color", "Size").
    pub name:              String,
    /// Attribute value.
    pub value:             String,
    /// Whether this affects pricing.
    pub affects_pricing:   bool,
    /// Whether this affects inventory.
    pub affects_inventory: bool,
}

impl ProductAttribute {
    /// Creates a new attribute.
    #[must_use]
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name:              name.into(),
            value:             value.into(),
            affects_pricing:   false,
            affects_inventory: false,
        }
    }
}

/// Product variant for configurable products.
#[derive(Debug, Clone)]
pub struct ProductVariant {
    /// Variant ID.
    pub id:              ProductId,
    /// Parent product ID.
    pub parent_id:       ProductId,
    /// Variant SKU.
    pub sku:             Sku,
    /// Attributes that define this variant.
    pub attributes:      Vec<ProductAttribute>,
    /// Variant-specific price (if different from parent).
    pub price_override:  Option<Price>,
    /// Variant-specific inventory count.
    pub inventory_count: i64,
    /// Whether variant is active.
    pub is_active:       bool,
}

impl ProductVariant {
    /// Creates a new variant.
    #[must_use]
    pub fn new(id: ProductId, parent_id: ProductId, sku: Sku) -> Self {
        Self {
            id,
            parent_id,
            sku,
            attributes: Vec::new(),
            price_override: None,
            inventory_count: 0,
            is_active: true,
        }
    }
}

// ============================================================================
// CATEGORY
// ============================================================================

/// Product category in the catalog hierarchy.
#[derive(Debug, Clone)]
pub struct Category {
    /// Category ID.
    pub id:               CategoryId,
    /// Category name.
    pub name:             String,
    /// Category description.
    pub description:      String,
    /// Parent category (if not root).
    pub parent_id:        Option<CategoryId>,
    /// URL slug for the category.
    pub slug:             String,
    /// Sort order within parent.
    pub sort_order:       u32,
    /// Whether category is visible.
    pub is_active:        bool,
    /// Category image URL.
    pub image_url:        Option<String>,
    /// SEO meta title.
    pub meta_title:       Option<String>,
    /// SEO meta description.
    pub meta_description: Option<String>,
}

impl Category {
    /// Creates a new category.
    #[must_use]
    pub fn new(id: CategoryId, name: impl Into<String>) -> Self {
        let name = name.into();
        let slug = name.to_lowercase().replace(' ', "-");
        Self {
            id,
            name,
            description: String::new(),
            parent_id: None,
            slug,
            sort_order: 0,
            is_active: true,
            image_url: None,
            meta_title: None,
            meta_description: None,
        }
    }

    /// Sets the parent category.
    #[must_use]
    pub fn with_parent(mut self, parent_id: CategoryId) -> Self {
        self.parent_id = Some(parent_id);
        self
    }
}

// ============================================================================
// PRODUCT
// ============================================================================

/// Complete product definition.
#[derive(Debug, Clone)]
pub struct Product {
    /// Product ID.
    pub id:                  ProductId,
    /// Product SKU.
    pub sku:                 Sku,
    /// Product name.
    pub name:                String,
    /// Product description.
    pub description:         String,
    /// Short description for listings.
    pub short_description:   String,
    /// Product type.
    pub product_type:        ProductType,
    /// Product status.
    pub status:              ProductStatus,
    /// Base price.
    pub price:               Price,
    /// Sale/promotional price.
    pub sale_price:          Option<Price>,
    /// Cost price (for profit calculation).
    pub cost_price:          Option<Price>,
    /// Category IDs.
    pub categories:          Vec<CategoryId>,
    /// Product images.
    pub images:              Vec<ProductImage>,
    /// Product attributes.
    pub attributes:          Vec<ProductAttribute>,
    /// Product variants.
    pub variants:            Vec<ProductVariant>,
    /// Physical dimensions.
    pub dimensions:          Option<ProductDimensions>,
    /// URL slug.
    pub slug:                String,
    /// SEO meta title.
    pub meta_title:          Option<String>,
    /// SEO meta description.
    pub meta_description:    Option<String>,
    /// Related product IDs.
    pub related_products:    Vec<ProductId>,
    /// Cross-sell product IDs.
    pub cross_sell_products: Vec<ProductId>,
    /// Tags for search.
    pub tags:                Vec<String>,
    /// Whether product is featured.
    pub is_featured:         bool,
    /// Whether product is taxable.
    pub is_taxable:          bool,
    /// Tax class identifier.
    pub tax_class:           Option<String>,
    /// Inventory quantity (for simple products).
    pub inventory_quantity:  i64,
    /// Low stock threshold.
    pub low_stock_threshold: u32,
    /// Whether backorders are allowed.
    pub backorders_allowed:  bool,
    /// Vendor/seller ID.
    pub vendor_id:           Option<String>,
    /// Creation timestamp.
    pub created_at:          u64,
    /// Last update timestamp.
    pub updated_at:          u64,
}

impl Product {
    /// Creates a new product.
    #[must_use]
    pub fn new(id: ProductId, sku: Sku, name: impl Into<String>) -> Self {
        let name = name.into();
        let slug = name.to_lowercase().replace(' ', "-");
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            id,
            sku,
            name,
            description: String::new(),
            short_description: String::new(),
            product_type: ProductType::Physical,
            status: ProductStatus::Draft,
            price: Price::default(),
            sale_price: None,
            cost_price: None,
            categories: Vec::new(),
            images: Vec::new(),
            attributes: Vec::new(),
            variants: Vec::new(),
            dimensions: None,
            slug,
            meta_title: None,
            meta_description: None,
            related_products: Vec::new(),
            cross_sell_products: Vec::new(),
            tags: Vec::new(),
            is_featured: false,
            is_taxable: true,
            tax_class: None,
            inventory_quantity: 0,
            low_stock_threshold: 10,
            backorders_allowed: false,
            vendor_id: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Gets the effective price (sale price if available).
    #[must_use]
    pub fn effective_price(&self) -> &Price {
        self.sale_price.as_ref().unwrap_or(&self.price)
    }

    /// Checks if product is on sale.
    #[must_use]
    pub fn is_on_sale(&self) -> bool {
        self.sale_price.is_some()
    }

    /// Checks if product is in stock.
    #[must_use]
    pub fn is_in_stock(&self) -> bool {
        self.inventory_quantity > 0 || self.backorders_allowed
    }

    /// Checks if product is low on stock.
    #[must_use]
    pub fn is_low_stock(&self) -> bool {
        self.inventory_quantity > 0
            && self.inventory_quantity <= i64::from(self.low_stock_threshold)
    }

    /// Gets the primary image.
    #[must_use]
    pub fn primary_image(&self) -> Option<&ProductImage> {
        self.images.iter().find(|img| img.is_primary).or_else(|| self.images.first())
    }

    /// Calculates profit margin.
    #[must_use]
    pub fn profit_margin(&self) -> Option<f64> {
        let cost = self.cost_price.as_ref()?;
        let price = self.effective_price();
        if cost.currency != price.currency {
            return None;
        }
        if cost.amount == 0 {
            return None;
        }
        Some((price.amount as f64 - cost.amount as f64) / price.amount as f64 * 100.0)
    }
}

// ============================================================================
// SEARCH & FILTERING
// ============================================================================

/// Search filters for product queries.
#[derive(Debug, Clone, Default)]
pub struct ProductFilter {
    /// Filter by category IDs.
    pub categories:    Vec<CategoryId>,
    /// Filter by status.
    pub status:        Option<ProductStatus>,
    /// Filter by product type.
    pub product_type:  Option<ProductType>,
    /// Minimum price filter.
    pub min_price:     Option<u64>,
    /// Maximum price filter.
    pub max_price:     Option<u64>,
    /// Filter by tags.
    pub tags:          Vec<String>,
    /// Filter by vendor ID.
    pub vendor_id:     Option<String>,
    /// Only featured products.
    pub featured_only: bool,
    /// Only in-stock products.
    pub in_stock_only: bool,
    /// Only products on sale.
    pub on_sale_only:  bool,
    /// Text search query.
    pub search_query:  Option<String>,
}

impl ProductFilter {
    /// Creates a new empty filter.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Filters by category.
    #[must_use]
    pub fn with_category(mut self, category_id: CategoryId) -> Self {
        self.categories.push(category_id);
        self
    }

    /// Filters by status.
    #[must_use]
    pub fn with_status(mut self, status: ProductStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Filters by price range.
    #[must_use]
    pub fn with_price_range(mut self, min: Option<u64>, max: Option<u64>) -> Self {
        self.min_price = min;
        self.max_price = max;
        self
    }

    /// Only in-stock products.
    #[must_use]
    pub fn in_stock_only(mut self) -> Self {
        self.in_stock_only = true;
        self
    }
}

/// Sort order for product listings.
#[derive(Debug, Clone, Copy, Default)]
pub enum ProductSortOrder {
    /// Sort by creation date, newest first.
    #[default]
    Newest,
    /// Sort by price, lowest first.
    PriceAsc,
    /// Sort by price, highest first.
    PriceDesc,
    /// Sort by name alphabetically.
    NameAsc,
    /// Sort by popularity/sales.
    BestSelling,
    /// Sort by rating.
    TopRated,
    /// Sort by featured status.
    Featured,
}

/// Paginated results.
#[derive(Debug, Clone)]
pub struct PaginatedProducts {
    /// Products in this page.
    pub products:    Vec<Product>,
    /// Total count of matching products.
    pub total_count: usize,
    /// Current page number (0-indexed).
    pub page:        usize,
    /// Items per page.
    pub page_size:   usize,
    /// Whether there are more pages.
    pub has_next:    bool,
}

impl PaginatedProducts {
    /// Total number of pages.
    #[must_use]
    pub fn total_pages(&self) -> usize {
        if self.page_size == 0 {
            return 0;
        }
        self.total_count.div_ceil(self.page_size)
    }
}

// ============================================================================
// PRODUCT CATALOG SERVICE
// ============================================================================

/// Product catalog management service.
#[derive(Debug)]
pub struct ProductCatalog {
    /// Products indexed by ID.
    products:          Arc<Mutex<HashMap<ProductId, Product>>>,
    /// Products indexed by SKU.
    products_by_sku:   Arc<Mutex<HashMap<Sku, ProductId>>>,
    /// Categories indexed by ID.
    categories:        Arc<Mutex<HashMap<CategoryId, Category>>>,
    /// Category hierarchy (parent -> children).
    category_children: Arc<Mutex<HashMap<CategoryId, Vec<CategoryId>>>>,
}

impl ProductCatalog {
    /// Creates a new product catalog.
    #[must_use]
    pub fn new() -> Self {
        Self {
            products:          Arc::new(Mutex::new(HashMap::new())),
            products_by_sku:   Arc::new(Mutex::new(HashMap::new())),
            categories:        Arc::new(Mutex::new(HashMap::new())),
            category_children: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    // ========================================================================
    // CATEGORY OPERATIONS
    // ========================================================================

    /// Adds a category to the catalog.
    ///
    /// # Errors
    /// Returns error if category ID already exists.
    pub fn add_category(&self, category: Category) -> Result<(), CommerceError> {
        let mut categories = self.categories.lock().map_err(|_| CommerceError::LockError)?;
        let mut children = self.category_children.lock().map_err(|_| CommerceError::LockError)?;

        if categories.contains_key(&category.id) {
            return Err(CommerceError::CategoryAlreadyExists(
                category.id.0.to_string(),
            ));
        }

        // Update parent's children list
        if let Some(parent_id) = &category.parent_id {
            children
                .entry(parent_id.clone())
                .or_insert_with(Vec::new)
                .push(category.id.clone());
        }

        categories.insert(category.id.clone(), category);
        Ok(())
    }

    /// Gets a category by ID.
    ///
    /// # Errors
    /// Returns error if category not found.
    pub fn get_category(&self, id: &CategoryId) -> Result<Category, CommerceError> {
        let categories = self.categories.lock().map_err(|_| CommerceError::LockError)?;
        categories
            .get(id)
            .cloned()
            .ok_or_else(|| CommerceError::CategoryNotFound(id.0.to_string()))
    }

    /// Gets all root categories.
    pub fn get_root_categories(&self) -> Result<Vec<Category>, CommerceError> {
        let categories = self.categories.lock().map_err(|_| CommerceError::LockError)?;
        Ok(categories.values().filter(|c| c.parent_id.is_none()).cloned().collect())
    }

    /// Gets child categories.
    pub fn get_child_categories(
        &self, parent_id: &CategoryId,
    ) -> Result<Vec<Category>, CommerceError> {
        let categories = self.categories.lock().map_err(|_| CommerceError::LockError)?;
        let children = self.category_children.lock().map_err(|_| CommerceError::LockError)?;

        let child_ids = children.get(parent_id).cloned().unwrap_or_default();
        Ok(child_ids.iter().filter_map(|id| categories.get(id).cloned()).collect())
    }

    // ========================================================================
    // PRODUCT OPERATIONS
    // ========================================================================

    /// Adds a product to the catalog.
    ///
    /// # Errors
    /// Returns error if product ID or SKU already exists.
    pub fn add_product(&self, product: Product) -> Result<(), CommerceError> {
        let mut products = self.products.lock().map_err(|_| CommerceError::LockError)?;
        let mut by_sku = self.products_by_sku.lock().map_err(|_| CommerceError::LockError)?;

        if products.contains_key(&product.id) {
            return Err(CommerceError::ProductAlreadyExists(
                product.id.0.to_string(),
            ));
        }

        if by_sku.contains_key(&product.sku) {
            return Err(CommerceError::SkuAlreadyExists(product.sku.0.to_string()));
        }

        by_sku.insert(product.sku.clone(), product.id.clone());
        products.insert(product.id.clone(), product);
        Ok(())
    }

    /// Gets a product by ID.
    ///
    /// # Errors
    /// Returns error if product not found.
    pub fn get_product(&self, id: &ProductId) -> Result<Product, CommerceError> {
        let products = self.products.lock().map_err(|_| CommerceError::LockError)?;
        products
            .get(id)
            .cloned()
            .ok_or_else(|| CommerceError::ProductNotFound(id.0.to_string()))
    }

    /// Gets a product by SKU.
    ///
    /// # Errors
    /// Returns error if product not found.
    pub fn get_product_by_sku(&self, sku: &Sku) -> Result<Product, CommerceError> {
        let by_sku = self.products_by_sku.lock().map_err(|_| CommerceError::LockError)?;
        let products = self.products.lock().map_err(|_| CommerceError::LockError)?;

        let id = by_sku
            .get(sku)
            .ok_or_else(|| CommerceError::ProductNotFound(sku.0.to_string()))?;
        products
            .get(id)
            .cloned()
            .ok_or_else(|| CommerceError::ProductNotFound(id.0.to_string()))
    }

    /// Updates a product.
    ///
    /// # Errors
    /// Returns error if product not found.
    pub fn update_product(&self, product: Product) -> Result<(), CommerceError> {
        let mut products = self.products.lock().map_err(|_| CommerceError::LockError)?;

        if !products.contains_key(&product.id) {
            return Err(CommerceError::ProductNotFound(product.id.0.to_string()));
        }

        products.insert(product.id.clone(), product);
        Ok(())
    }

    /// Removes a product.
    ///
    /// # Errors
    /// Returns error if product not found.
    pub fn remove_product(&self, id: &ProductId) -> Result<Product, CommerceError> {
        let mut products = self.products.lock().map_err(|_| CommerceError::LockError)?;
        let mut by_sku = self.products_by_sku.lock().map_err(|_| CommerceError::LockError)?;

        let product = products
            .remove(id)
            .ok_or_else(|| CommerceError::ProductNotFound(id.0.to_string()))?;
        by_sku.remove(&product.sku);
        Ok(product)
    }

    /// Searches products with filters.
    pub fn search_products(
        &self, filter: &ProductFilter, sort: ProductSortOrder, page: usize, page_size: usize,
    ) -> Result<PaginatedProducts, CommerceError> {
        let products = self.products.lock().map_err(|_| CommerceError::LockError)?;

        // Filter products
        let mut filtered: Vec<Product> =
            products.values().filter(|p| self.matches_filter(p, filter)).cloned().collect();

        let total_count = filtered.len();

        // Sort products
        self.sort_products(&mut filtered, sort);

        // Paginate
        let start = page * page_size;
        let end = (start + page_size).min(filtered.len());
        let page_products = if start < filtered.len() {
            filtered[start..end].to_vec()
        } else {
            Vec::new()
        };

        Ok(PaginatedProducts {
            products: page_products,
            total_count,
            page,
            page_size,
            has_next: end < total_count,
        })
    }

    /// Gets products in a category.
    pub fn get_products_by_category(
        &self, category_id: &CategoryId, include_subcategories: bool,
    ) -> Result<Vec<Product>, CommerceError> {
        let products = self.products.lock().map_err(|_| CommerceError::LockError)?;

        let category_ids = if include_subcategories {
            self.get_descendant_categories(category_id)?
        } else {
            vec![category_id.clone()]
        };

        Ok(products
            .values()
            .filter(|p| p.categories.iter().any(|c| category_ids.contains(c)))
            .cloned()
            .collect())
    }

    /// Gets featured products.
    pub fn get_featured_products(&self, limit: usize) -> Result<Vec<Product>, CommerceError> {
        let products = self.products.lock().map_err(|_| CommerceError::LockError)?;

        let mut featured: Vec<_> = products
            .values()
            .filter(|p| p.is_featured && p.status.is_visible())
            .cloned()
            .collect();

        featured.truncate(limit);
        Ok(featured)
    }

    /// Gets products on sale.
    pub fn get_sale_products(&self, limit: usize) -> Result<Vec<Product>, CommerceError> {
        let products = self.products.lock().map_err(|_| CommerceError::LockError)?;

        let mut on_sale: Vec<_> = products
            .values()
            .filter(|p| p.is_on_sale() && p.status.is_visible())
            .cloned()
            .collect();

        on_sale.truncate(limit);
        Ok(on_sale)
    }

    /// Gets related products.
    pub fn get_related_products(
        &self, product_id: &ProductId,
    ) -> Result<Vec<Product>, CommerceError> {
        let product = self.get_product(product_id)?;
        let products = self.products.lock().map_err(|_| CommerceError::LockError)?;

        Ok(product
            .related_products
            .iter()
            .filter_map(|id| products.get(id).cloned())
            .collect())
    }

    // ========================================================================
    // PRIVATE HELPERS
    // ========================================================================

    /// Checks if product matches filter.
    fn matches_filter(&self, product: &Product, filter: &ProductFilter) -> bool {
        // Status filter
        if filter.status.is_some_and(|status| product.status != status) {
            return false;
        }

        // Product type filter
        if filter.product_type.is_some_and(|pt| product.product_type != pt) {
            return false;
        }

        // Category filter
        if !filter.categories.is_empty()
            && !filter.categories.iter().any(|c| product.categories.contains(c))
        {
            return false;
        }

        // Price range filter
        let price = product.effective_price().amount;
        if filter.min_price.is_some_and(|min| price < min) {
            return false;
        }
        if filter.max_price.is_some_and(|max| price > max) {
            return false;
        }

        // Tags filter
        if !filter.tags.is_empty() && !filter.tags.iter().any(|t| product.tags.contains(t)) {
            return false;
        }

        // Vendor filter
        if filter
            .vendor_id
            .as_ref()
            .is_some_and(|vendor_id| product.vendor_id.as_ref() != Some(vendor_id))
        {
            return false;
        }

        // Featured filter
        if filter.featured_only && !product.is_featured {
            return false;
        }

        // In-stock filter
        if filter.in_stock_only && !product.is_in_stock() {
            return false;
        }

        // On-sale filter
        if filter.on_sale_only && !product.is_on_sale() {
            return false;
        }

        // Text search
        if let Some(query) = &filter.search_query {
            let query_lower = query.to_lowercase();
            let matches_name = product.name.to_lowercase().contains(&query_lower);
            let matches_desc = product.description.to_lowercase().contains(&query_lower);
            let matches_sku = product.sku.0.to_lowercase().contains(&query_lower);
            if !matches_name && !matches_desc && !matches_sku {
                return false;
            }
        }

        true
    }

    /// Sorts products by specified order.
    fn sort_products(&self, products: &mut [Product], sort: ProductSortOrder) {
        match sort {
            ProductSortOrder::Newest => {
                products.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            },
            ProductSortOrder::PriceAsc => {
                products
                    .sort_by(|a, b| a.effective_price().amount.cmp(&b.effective_price().amount));
            },
            ProductSortOrder::PriceDesc => {
                products
                    .sort_by(|a, b| b.effective_price().amount.cmp(&a.effective_price().amount));
            },
            ProductSortOrder::NameAsc => {
                products.sort_by(|a, b| a.name.cmp(&b.name));
            },
            ProductSortOrder::BestSelling | ProductSortOrder::TopRated => {
                // Would require sales/rating data - for now, sort by created date
                products.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            },
            ProductSortOrder::Featured => {
                products.sort_by(|a, b| b.is_featured.cmp(&a.is_featured));
            },
        }
    }

    /// Gets all descendant category IDs.
    fn get_descendant_categories(
        &self, category_id: &CategoryId,
    ) -> Result<Vec<CategoryId>, CommerceError> {
        let children = self.category_children.lock().map_err(|_| CommerceError::LockError)?;

        let mut result = vec![category_id.clone()];
        let mut to_process = vec![category_id.clone()];

        while let Some(current) = to_process.pop() {
            if let Some(child_ids) = children.get(&current) {
                for child_id in child_ids {
                    result.push(child_id.clone());
                    to_process.push(child_id.clone());
                }
            }
        }

        Ok(result)
    }
}

impl Default for ProductCatalog {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_product_creation() {
        let product = Product::new(
            ProductId::new("prod-001"),
            Sku::new("SKU-001"),
            "Test Product",
        );

        assert_eq!(product.id.as_str(), "prod-001");
        assert_eq!(product.name, "Test Product");
        assert_eq!(product.status, ProductStatus::Draft);
    }

    #[test]
    fn test_price_operations() {
        let price1 = Price::new(1000, Currency::usd(), 2);
        let price2 = Price::new(500, Currency::usd(), 2);

        let total = price1.add(&price2).expect("should add prices");
        assert_eq!(total.amount, 1500);
        assert_eq!(total.display_amount(), 15.0);
    }

    #[test]
    fn test_catalog_add_product() {
        let catalog = ProductCatalog::new();
        let product = Product::new(
            ProductId::new("prod-001"),
            Sku::new("SKU-001"),
            "Test Product",
        );

        catalog.add_product(product).expect("should add product");

        let retrieved =
            catalog.get_product(&ProductId::new("prod-001")).expect("should get product");
        assert_eq!(retrieved.name, "Test Product");
    }

    #[test]
    fn test_catalog_duplicate_sku() {
        let catalog = ProductCatalog::new();

        let product1 = Product::new(ProductId::new("prod-001"), Sku::new("SKU-001"), "Product 1");
        let product2 = Product::new(
            ProductId::new("prod-002"),
            Sku::new("SKU-001"), // Same SKU
            "Product 2",
        );

        catalog.add_product(product1).expect("should add first product");
        let result = catalog.add_product(product2);
        assert!(result.is_err());
    }

    #[test]
    fn test_category_hierarchy() {
        let catalog = ProductCatalog::new();

        let root = Category::new(CategoryId::new("cat-root"), "Electronics");
        let child = Category::new(CategoryId::new("cat-phones"), "Phones")
            .with_parent(CategoryId::new("cat-root"));

        catalog.add_category(root).expect("should add root");
        catalog.add_category(child).expect("should add child");

        let children = catalog
            .get_child_categories(&CategoryId::new("cat-root"))
            .expect("should get children");
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].name, "Phones");
    }

    #[test]
    fn test_product_search() {
        let catalog = ProductCatalog::new();

        let mut product1 =
            Product::new(ProductId::new("prod-001"), Sku::new("SKU-001"), "iPhone 15");
        product1.status = ProductStatus::Active;
        product1.price = Price::new(99900, Currency::usd(), 2);

        let mut product2 = Product::new(
            ProductId::new("prod-002"),
            Sku::new("SKU-002"),
            "Samsung Galaxy",
        );
        product2.status = ProductStatus::Active;
        product2.price = Price::new(79900, Currency::usd(), 2);

        catalog.add_product(product1).expect("add product1");
        catalog.add_product(product2).expect("add product2");

        let filter = ProductFilter::new().with_status(ProductStatus::Active);
        let results = catalog
            .search_products(&filter, ProductSortOrder::PriceAsc, 0, 10)
            .expect("search should succeed");

        assert_eq!(results.total_count, 2);
        assert_eq!(results.products[0].name, "Samsung Galaxy"); // Lower price first
    }

    #[test]
    fn test_effective_price() {
        let mut product = Product::new(
            ProductId::new("prod-001"),
            Sku::new("SKU-001"),
            "Test Product",
        );
        product.price = Price::new(10000, Currency::usd(), 2);

        assert_eq!(product.effective_price().amount, 10000);
        assert!(!product.is_on_sale());

        product.sale_price = Some(Price::new(7500, Currency::usd(), 2));
        assert_eq!(product.effective_price().amount, 7500);
        assert!(product.is_on_sale());
    }

    #[test]
    fn test_product_status() {
        assert!(ProductStatus::Active.is_visible());
        assert!(ProductStatus::Active.is_purchasable());
        assert!(ProductStatus::OutOfStock.is_visible());
        assert!(!ProductStatus::OutOfStock.is_purchasable());
        assert!(!ProductStatus::Draft.is_visible());
    }
}
