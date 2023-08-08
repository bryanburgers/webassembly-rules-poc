package main;

func main() {}

//export validate
func validate() {
	var listing, previousListing Listing;
	data(&listing);
	previousData(&previousListing);

	if listing.ListPrice <= 0 {
		error("ListPrice", "List Price must be greater than $0");
	}
	if listing.MlsStatus == "Closed" {
		set_required("ClosePrice", true);
		set_display("ClosePrice", true);
	} else {
		set_required("ClosePrice", false);
		set_display("ClosePrice", false);
		set("ClosePrice", nil);
	}
}

type Listing struct {
    ListPrice int64
    MlsStatus string
}
