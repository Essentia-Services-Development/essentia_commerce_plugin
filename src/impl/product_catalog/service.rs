//! # Product Catalog Service (GAP-220-D-001)
//!
//! Service implementation for product catalog management.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    errors::CommerceError,
    types::product_catalog::{
        Category, CategoryId, PaginatedProducts, Product, ProductFilter, ProductId,
        ProductSortOrder, Sku,
    },
};

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
